mod detect;
mod trade;

use anyhow::Result;
use detect::Detector;
use futures_util::StreamExt;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_response::RpcLogsResponse;
use solana_sdk::hash::Hash;
use solana_sdk::signature::{read_keypair_file, Signer};
use std::sync::{Arc, RwLock};
use trade::{Priority, build_buy_ixs, fast_send};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // endpoints (testnet)
    let ws_url = "wss://api.testnet.solana.com";
    let rpc_url = "https://api.testnet.solana.com";

    // Create a shared RpcClient wrapped in Arc
    let rpc = Arc::new(RpcClient::new(rpc_url.to_string()));

    // Load wallet keypair
    let keypath = std::env::var("WALLET_KEYPATH")
        .unwrap_or_else(|_| "~/.config/solana/id.json".to_string());
    let keypath = shellexpand::tilde(&keypath).to_string();
    let payer = Arc::new(
        read_keypair_file(&keypath)
            .map_err(|e| anyhow::anyhow!("failed to read keypair file {}: {:?}", keypath, e))?,
    );

    // Priority config
    let prio = Priority {
        compute_unit_limit: std::env::var("COMPUTE_UNIT_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_000_000),
        microlamports_per_cu: std::env::var("PRIORITY_MICROLAMPORTS_PER_CU")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5_000),
    };

    // Shared recent blockhash
    let initial_bh = rpc
        .as_ref()
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow::anyhow!("failed to get initial blockhash: {:?}", e))?;
    let bhash: Arc<RwLock<Hash>> = Arc::new(RwLock::new(initial_bh));

    // Spawn a task to refresh the blockhash
    {
        let rpc_bh = Arc::clone(&rpc);
        let bhash2 = Arc::clone(&bhash);
        tokio::spawn(async move {
            loop {
                match rpc_bh.get_latest_blockhash().await {
                    Ok(h) => {
                        let mut w = bhash2.write().unwrap();
                        *w = h;
                    }
                    Err(e) => {
                        eprintln!("blockhash refresh err: {:?}", e);
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });
    }

    // Detector setup
    let detector = Detector::new(
        std::env::var("RAYDIUM_AMM_V4").ok().as_deref(),
        std::env::var("RAYDIUM_CLMM").ok().as_deref(),
        std::env::var("PUMP_FUN").ok().as_deref(),
    );

    // Connect to PubsubClient
    let client = PubsubClient::new(ws_url)
        .await
        .map_err(|e| anyhow::anyhow!("failed to connect to websocket {}: {:?}", ws_url, e))?;

    let config = RpcTransactionLogsConfig { commitment: None };

    // Subscribe to logs
    let (mut logs_stream, _unsubscribe) = client
        .logs_subscribe(RpcTransactionLogsFilter::All, config)
        .await
        .map_err(|e| anyhow::anyhow!("failed to subscribe to logs: {:?}", e))?;

    println!("📡 Listening to Solana Testnet logs...");

    // Process transaction logs
    while let Some(notif) = logs_stream.next().await {
        // notif is already Response<RpcLogsResponse>
        let slot = notif.context.slot;
        let logs: &RpcLogsResponse = &notif.value;

        if let Some(evt) = detector.process(logs, slot) {
            println!(
                "[DETECT] {:?} at slot {} sig={}",
                evt.kind, evt.detected_at_slot, evt.signature
            );

            if let Some(token_mint) = evt.token_mint {
                let amount_sol: u64 = std::env::var("BASE_BUY_SOL")
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.02)
                    .mul_add(1_000_000_000f64, 0.0) as u64;

                let slippage_bps: u16 = std::env::var("SLIPPAGE_BPS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(500u16);

                match build_buy_ixs(
                    rpc.as_ref(),
                    &payer.pubkey(),
                    &token_mint,
                    amount_sol,
                    slippage_bps,
                )
                .await
                {
                    Ok(ixs) => {
                        let sig = fast_send(
                            rpc.as_ref(),
                            &payer,
                            ixs,
                            &*bhash.read().unwrap(),
                            &prio,
                            true,
                        )
                        .await;
                        println!("[BUY] mint={} sig={:?}", token_mint, sig);
                    }
                    Err(e) => eprintln!("[BUY-ERR] {:?}", e),
                }
            } else {
                eprintln!("[DETECT] token mint not extracted, skipping buy");
            }
        }
    }

    Ok(())
}

// src/main.rs

mod detect;
mod trade;

use detect::Detector;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsConfig;
use solana_client::rpc_response::RpcLogsResponse; // FIX: RpcTransactionLogs → RpcLogsResponse
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::{read_keypair_file, Signer}; // FIX: add Signer trait for pubkey()
use solana_sdk::hash::Hash;
use trade::{Priority, build_buy_ixs, fast_send};
use futures_util::StreamExt;
use std::sync::{Arc, RwLock};
use tokio;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let ws_url = "wss://api.testnet.solana.com";
    let rpc_url = "https://api.testnet.solana.com";

    // Create RPC client wrapped in Arc
    let rpc = Arc::new(RpcClient::new(rpc_url.to_string()));

    // Load wallet keypair
    let keypath = std::env::var("WALLET_KEYPATH")?;
    let payer = Arc::new(read_keypair_file(&keypath)?);

    // Priority config
    let prio = Priority {
        compute_unit_limit: std::env::var("COMPUTE_UNIT_LIMIT")?.parse().unwrap_or(1_000_000),
        microlamports_per_cu: std::env::var("PRIORITY_MICROLAMPORTS_PER_CU")?.parse().unwrap_or(5_000),
    };

    // Shared recent blockhash
    let bhash: Arc<RwLock<Hash>> = Arc::new(RwLock::new(rpc.get_latest_blockhash().await?));

    // Spawn task to refresh blockhash every 3 seconds
    {
        let rpc_bh = Arc::clone(&rpc); // FIX: use Arc::clone
        let bhash2 = bhash.clone();
        tokio::spawn(async move {
            loop {
                match rpc_bh.get_latest_blockhash().await {
                    Ok(h) => { *bhash2.write().unwrap() = h; }
                    Err(e) => eprintln!("blockhash refresh err: {e:?}"),
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
        .expect("Failed to connect to Solana WebSocket");

    let config = RpcTransactionLogsConfig { commitment: None };

    let (mut logs_stream, _unsubscribe) = client
        .logs_subscribe(solana_client::rpc_config::RpcTransactionLogsFilter::All, config)
        .await
        .expect("Failed to subscribe to logs");

    println!("📡 Listening to Solana Testnet logs...");

    // Process transaction logs
    while let Some(msg) = logs_stream.next().await {
        if let Ok(notif) = msg {
            if let Some(evt) = detector.process(&notif.value) { // notif.value is RpcLogsResponse
                println!(
                    "[DETECT] {:?} at slot {} sig={}",
                    evt.kind, evt.detected_at_slot, evt.signature
                );

                // Auto-buy logic
                let token_mint = evt.token_mint;
                let amount_sol: u64 = ((std::env::var("BASE_BUY_SOL")?
                    .parse::<f64>()
                    .unwrap_or(0.02)) * 1_000_000_000f64) as u64;
                let slippage_bps: u16 = std::env::var("SLIPPAGE_BPS")?.parse().unwrap_or(500);

                match build_buy_ixs(&rpc, &payer.pubkey(), &token_mint, amount_sol, slippage_bps).await {
                    Ok(ixs) => {
                        let sig = fast_send(&rpc, &payer, ixs, &*bhash.read().unwrap(), &prio, true).await;
                        println!("[BUY] mint={} sig={:?}", token_mint, sig);
                    }
                    Err(e) => eprintln!("[BUY-ERR] {e:?}"),
                }
            }
        }
    }

    Ok(())
}

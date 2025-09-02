// src/main.rs

use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use futures_util::StreamExt;
use tokio;

#[tokio::main]
async fn main() {
    let ws_url = "wss://api.testnet.solana.com";

    // Connect to PubsubClient
    let client = PubsubClient::new(ws_url)
        .await
        .expect("Failed to connect to Solana Testnet WebSocket");

    // Construct logs config
    let config = RpcTransactionLogsConfig {
        commitment: None, // or Some(solana_client::rpc_config::CommitmentConfig::confirmed())
    };

    // Subscribe to all transaction logs
    let (mut logs_sub, _unsubscribe) = client
        .logs_subscribe(RpcTransactionLogsFilter::All, config)
        .await
        .expect("Failed to subscribe to logs");

    println!("📡 Listening to Solana Testnet logs...");

    while let Some(response) = logs_sub.next().await {
        // response.value has signature and logs
        let value = response.value;
        println!("✅ Transaction: {}", value.signature);
        for log_line in value.logs {
            println!("   {}", log_line);
        }
    }
}

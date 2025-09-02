use serde::{Deserialize, Serialize};
use solana_client::rpc_response::RpcTransactionLogs;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaunchKind { RaydiumAMM, RaydiumCLMM, PumpFun }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchEvent {
    pub kind: LaunchKind,
    pub signature: String,
    pub token_mint: Pubkey,
    pub base_is_sol: bool,
    pub detected_at_slot: u64,
}

pub struct Detector {
    raydium_amm: Option<Pubkey>,
    raydium_clmm: Option<Pubkey>,
    pump_fun: Option<Pubkey>,
}

impl Detector {
    pub fn new(raydium_amm: Option<&str>, raydium_clmm: Option<&str>, pump_fun: Option<&str>) -> Self {
        Self {
            raydium_amm: raydium_amm.and_then(|s| s.parse().ok()),
            raydium_clmm: raydium_clmm.and_then(|s| s.parse().ok()),
            pump_fun: pump_fun.and_then(|s| s.parse().ok()),
        }
    }

    pub fn process(&self, logs: &RpcTransactionLogs) -> Option<LaunchEvent> {
        let sig = logs.signature.clone();
        let slot = logs.context.slot;
        let mentions = &logs.value;
        let text = mentions.logs.join("\n");

        // Very simple heuristics — you’ll tighten these with real patterns.
        if let Some(pid) = &self.raydium_amm {
            if mentions.mentions.iter().any(|m| m == &pid.to_string()) && text.to_lowercase().contains("initialize") {
                // TODO: parse token mint from accounts or log text
                // Placeholder: we can’t extract without decoding inner instructions here.
                return None;
            }
        }

        if let Some(pid) = &self.pump_fun {
            if mentions.mentions.iter().any(|m| m == &pid.to_string()) && text.to_lowercase().contains("initialize") {
                // TODO: extract mint
                return None;
            }
        }
        None
    }
}

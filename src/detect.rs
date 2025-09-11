use serde::{Deserialize, Serialize};
use solana_client::rpc_response::RpcLogsResponse;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaunchKind {
    RaydiumAMM,
    RaydiumCLMM,
    PumpFun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchEvent {
    pub kind: LaunchKind,
    pub signature: String,
    pub token_mint: Option<Pubkey>,
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

    /// Process RpcLogsResponse + slot into a LaunchEvent
    pub fn process(&self, logs: &RpcLogsResponse, slot: u64) -> Option<LaunchEvent> {
        let sig = logs.signature.clone();
        let text = logs.logs.join("\n").to_lowercase();

        if let Some(pid) = &self.raydium_amm {
            if text.contains("initialize") {
                return Some(LaunchEvent {
                    kind: LaunchKind::RaydiumAMM,
                    signature: sig,
                    token_mint: None,
                    base_is_sol: true,
                    detected_at_slot: slot,
                });
            }
        }

        if let Some(pid) = &self.raydium_clmm {
            if text.contains("initialize") {
                return Some(LaunchEvent {
                    kind: LaunchKind::RaydiumCLMM,
                    signature: sig,
                    token_mint: None,
                    base_is_sol: true,
                    detected_at_slot: slot,
                });
            }
        }

        if let Some(pid) = &self.pump_fun {
            if text.contains("initialize") {
                return Some(LaunchEvent {
                    kind: LaunchKind::PumpFun,
                    signature: sig,
                    token_mint: None,
                    base_is_sol: true,
                    detected_at_slot: slot,
                });
            }
        }

        None
    }
}

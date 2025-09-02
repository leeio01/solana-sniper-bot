use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub mint: Pubkey,
    pub buy_sig: String,
    pub buy_price: f64,    // in SOL per token
    pub amount_tokens: f64,
    pub opened_at: DateTime<Utc>,
    pub take_profit_pct: f64,
    pub stop_loss_pct: f64,
    pub max_seconds: i64,
}

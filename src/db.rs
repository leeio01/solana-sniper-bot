// src/db.rs

use anyhow::Result;
use chrono::Utc;
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub ts: i64,
    pub side: String,      // BUY or SELL
    pub mint: String,
    pub signature: String,
    pub qty: f64,
    pub price_sol: f64,    // SOL per token
}

#[derive(Clone)]
pub struct DB {
    pub collection: Collection<Trade>,
}

impl DB {
    /// Connect to MongoDB and get "trades" collection
    pub async fn open(uri: &str, db_name: &str, coll_name: &str) -> Result<Self> {
        let client_options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(client_options)?;
        let collection = client.database(db_name).collection::<Trade>(coll_name);
        Ok(Self { collection })
    }

    /// Log a trade to MongoDB
    pub async fn log_trade(&self, side: &str, mint: &str, signature: &str, qty: f64, price_sol: f64) -> Result<()> {
        let trade = Trade {
            ts: Utc::now().timestamp(),
            side: side.to_string(),
            mint: mint.to_string(),
            signature: signature.to_string(),
            qty,
            price_sol,
        };
        self.collection.insert_one(trade, None).await?;
        Ok(())
    }

    /// fetch all trades
    pub async fn fetch_trades(&self) -> Result<Vec<Trade>> {
        let cursor = self.collection.find(None, None).await?;
        let trades: Vec<Trade> = cursor.try_collect().await?;
        Ok(trades)
    }
}

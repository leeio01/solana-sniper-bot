use anyhow::Result;
use rusqlite::{Connection, params};

pub struct DB { conn: Connection }

impl DB {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts INTEGER NOT NULL,
                side TEXT NOT NULL,       -- BUY or SELL
                mint TEXT NOT NULL,
                signature TEXT NOT NULL,
                qty REAL NOT NULL,
                price_sol REAL NOT NULL   -- SOL per token
            );
        "#)?;
        Ok(Self { conn })
    }

    pub fn log_trade(&self, side: &str, mint: &str, signature: &str, qty: f64, price_sol: f64) -> Result<()> {
        let ts = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO trades(ts, side, mint, signature, qty, price_sol) VALUES(?,?,?,?,?,?)",
            params![ts, side, mint, signature, qty, price_sol],
        )?;
        Ok(())
    }
}

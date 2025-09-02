use anyhow::{Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

/// Build Raydium/Jupiter TOKEN->SOL swap ixs; placeholder
pub async fn build_sell_ixs(
    _rpc: &RpcClient,
    _owner: &Pubkey,
    _token_mint: &Pubkey,
    _amount_tokens: u64,
    _slippage_bps: u16,
) -> Result<Vec<Instruction>> {
    Err(anyhow!("sell not implemented yet"))
}

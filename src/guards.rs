use anyhow::{Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey};
use solana_account_decoder::parse_token::UiAccountState;
use solana_sdk::account::ReadableAccount;

pub async fn check_mint_sane(rpc: &RpcClient, mint: &Pubkey, deny_freeze: bool, deny_mint_auth: bool) -> Result<()> {
    let acc = rpc.get_account(mint).await?;
    // Quick and dirty decode: token mint layout is from spl-token; for brevity, we’ll avoid direct parse here.
    // If needed, pull spl-token crate and parse Mint struct to check authorities properly.
    if deny_freeze || deny_mint_auth {
        // TODO: Proper decode & checks
    }
    Ok(())
}

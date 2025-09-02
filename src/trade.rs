use anyhow::{Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    signature::{Keypair, Signer},
    transaction::Transaction,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    pubkey::Pubkey,
};
use std::time::Duration;

pub struct Priority {
    pub compute_unit_limit: u32,
    pub microlamports_per_cu: u64,
}

pub async fn fast_send(
    rpc: &RpcClient,
    payer: &Keypair,
    ixs: Vec<Instruction>,
    blockhash: &Hash,
    prio: &Priority,
    skip_preflight: bool,
) -> Result<String> {
    let mut full_ixs = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(prio.compute_unit_limit),
        ComputeBudgetInstruction::set_compute_unit_price(prio.microlamports_per_cu),
    ];
    full_ixs.extend(ixs);

    let msg = Message::new(&full_ixs, Some(&payer.pubkey()));
    let mut tx = Transaction::new_unsigned(msg);
    tx.try_sign(&[payer], *blockhash)?;

    let sig = if skip_preflight {
        rpc.send_transaction_with_config(
            &tx,
            solana_client::rpc_config::RpcSendTransactionConfig {
                skip_preflight: true,
                preflight_commitment: Some(CommitmentConfig::processed().commitment),
                max_retries: Some(3),
                ..Default::default()
            }
        ).await?
    } else {
        rpc.send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            CommitmentConfig::processed(),
            solana_client::rpc_config::RpcSendTransactionConfig {
                max_retries: Some(3),
                ..Default::default()
            }
        ).await?
    };

    Ok(sig.to_string())
}

/// Stub for building a BUY swap on Raydium/Jupiter. Fill actual route later.
pub async fn build_buy_ixs(
    _rpc: &RpcClient,
    _payer: &Pubkey,
    _token_mint: &Pubkey,
    _amount_sol: u64,
    _slippage_bps: u16,
) -> Result<Vec<Instruction>> {
    // TODO:
    // 1) create ATA if missing
    // 2) wrap SOL -> WSOL (if needed)
    // 3) swap SOL->TOKEN via Raydium (constant product) or Jupiter route
    Err(anyhow!("buy route not implemented yet"))
}

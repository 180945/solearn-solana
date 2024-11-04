use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 8 + PoolInfo::LEN)]
    pub pool_info: Account<'info, PoolInfo>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MinerRegister<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    #[account(init, payer = user, space = 8 + MinerInfo::LEN)]
    pub miner_info: Account<'info, UserInfo>,
    #[account(mut)]
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == sol_learn_account.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    /// CHECK:
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    #[account(mut)]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    /// CHECK:
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub admin: AccountInfo<'info>,
    #[account(mut)]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
pub struct PoolInfo {
    pub admin: Pubkey,
    pub start_slot: u64,
    pub end_slot: u64,
    pub token: Pubkey,
}

#[account]
pub struct UserInfo {
    pub amount: u64,
    pub reward_debt: u64,
    pub deposit_slot: u64,
}

impl UserInfo {
    pub const LEN: usize = 8 + 8 + 8;
}

impl PoolInfo {
    pub const LEN: usize = 32 + 8 + 8 + 32;
}

// Contract info
#[account]
pub struct SolLearnInfo {
    pub owner: Pubkey,
    pub token: Pubkey,
    pub total_miner: u64,
    pub total_models: u64,
    pub total_infer: u64,
    pub miner_min_stake: u64,
}

#[account]
pub struct MinerInfo {
    pub miner: Pubkey,
    pub stake_amount: u64,
    pub last_epoch: u64,
}

impl MinerInfo {
    pub const LEN: usize = 32 + 8 + 8 + 32;
}
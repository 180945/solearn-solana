use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{DAOTokenPercentage, Tasks};

// init pda to store list of models
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    // #[account(init, payer = admin, space = 8 + SolLearnInfo::LEN)]
    // pub sol_learn_account: Account<'info, SolLearnInfo>,
    pub staking_token: InterfaceAccount<'info, Mint>,
    #[account(
        init, 
        payer = admin, 
        space = 8 + VaultAccount::LEN,
        seeds = [b"vault", sol_learn_account.key().as_ref()], 
        bump
    )]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(
        init, 
        payer = admin, 
        space = 8 + Models::LEN,
        seeds = [b"models", sol_learn_account.key().as_ref()], 
        bump
    )]
    pub models: Box<Account<'info, Models>>,
    #[account(
        init, 
        // realloc = 8 + wh_account.len(),
        // realloc::payer = admin, 
        // realloc::zero = false,
        payer = admin, 
        space = 8 + SolLearnInfo::LEN,
    )]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    pub system_program: Program<'info, System>,
    pub sysvar_clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct InitializeExtra<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    // pub tasks: Box<Account<'info, Tasks>>,
    #[account(mut, constraint = sol_learn_account.admin == admin.key())]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(model: Pubkey)]
pub struct AddModel<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK:
    #[account(mut, constraint = sol_learn_account.admin == admin.key())]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    #[account(
        mut,
        realloc = 8 + Models::LEN + models.data.len() + 32,
        realloc::payer = admin,
        realloc::zero = false,
        // seeds = [b"models", sol_learn_account.key().as_ref()], 
        // bump = models.bump,
    )]
    pub models: Box<Account<'info, Models>>,
    #[account(
        init, 
        payer = admin, 
        space = 8 + MinersOfModel::LEN,
        seeds = [b"models", sol_learn_account.key().as_ref(), model.key().as_ref()], 
        bump
    )]
    pub miners_of_model: Box<Account<'info, MinersOfModel>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(model: Pubkey)]
pub struct RemoveModel<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK:
    #[account(mut, constraint = sol_learn_account.admin == admin.key())]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    #[account(
        mut,
        realloc = 8 + 1 + 4 + 8 + 8 + models.data.len() + 32,
        realloc::payer = admin,
        realloc::zero = false,
        // seeds = [b"models", sol_learn_account.key().as_ref()], 
        // bump = models.bump,
    )]
    pub models: Box<Account<'info, Models>>,
    #[account(
        mut, 
        close = admin, 
        seeds = [b"models", sol_learn_account.key().as_ref(), model.key().as_ref()], 
        bump = miners_of_model.bump
    )]
    pub miners_of_model: Box<Account<'info, MinersOfModel>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MinerRegister<'info> {
    #[account(mut)]
    pub miner: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    #[account(
        // seeds = [b"models", sol_learn_account.key().as_ref()], 
        // bump = models.bump,
    )]
    pub models: Box<Account<'info, Models>>,
    #[account(
        init, 
        payer = miner, 
        space = 8 + MinerInfo::LEN,
        seeds = [b"miner", miner.key().as_ref(), sol_learn_account.key().as_ref()], 
        bump,
    )]
    pub miner_account: Account<'info, MinerInfo>,
    #[account(mut)]
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"vault", sol_learn_account.key().as_ref()], 
        bump = vault_wallet_owner_pda.bump,
    )]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub sysvar_clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Topup<'info> {
    /// CHECK:
    #[account(mut)]
    pub miner: Signer<'info>,
    /// CHECK:
    #[account()]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    #[account(
        mut,
        seeds = [b"miner", miner.key().as_ref(), sol_learn_account.key().as_ref()], 
        bump = miner_account.bump,
    )]
    pub miner_account: Account<'info, MinerInfo>,
    #[account(mut)]
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"vault", sol_learn_account.key().as_ref()], 
        bump = vault_wallet_owner_pda.bump,
    )]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct MinerUnStaking<'info> {
    #[account(mut)]
    pub miner: Signer<'info>,
    /// CHECK:
    #[account()]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    #[account(
        mut,
        seeds = [b"miner", miner.key().as_ref(), sol_learn_account.key().as_ref()], 
        bump = miner_account.bump,
    )]
    pub miner_account: Account<'info, MinerInfo>,
    #[account(
        mut,
        seeds = [b"models", sol_learn_account.key().as_ref(), miner_account.model.key().as_ref()], 
        bump = miners_of_model.bump,
    )]
    pub miners_of_model: Box<Account<'info, MinersOfModel>>,
    pub system_program: Program<'info, System>,
    pub sysvar_clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct MinerClaimReward<'info> {
    #[account(mut)]
    pub miner: Signer<'info>,
    /// CHECK:
    #[account()]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    #[account(
        mut,
        seeds = [b"miner", miner.key().as_ref(), sol_learn_account.key().as_ref()], 
        bump = miner_account.bump,
    )]
    pub miner_account: Account<'info, MinerInfo>,
    #[account(mut)]
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"vault", sol_learn_account.key().as_ref()], 
        bump = vault_wallet_owner_pda.bump,
    )]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub sysvar_clock: Sysvar<'info, Clock>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct MinerClaim<'info> {
    #[account(mut)]
    pub miner: Signer<'info>,
    /// CHECK:
    #[account()]
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    #[account(
        mut,
        seeds = [b"miner", miner.key().as_ref(), sol_learn_account.key().as_ref()], 
        bump = miner_account.bump,
    )]
    pub miner_account: Account<'info, MinerInfo>,
    #[account(mut)]
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"vault", sol_learn_account.key().as_ref()], 
        bump = vault_wallet_owner_pda.bump,
    )]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub sysvar_clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinForMinting<'info> {
    #[account(mut)]
    pub miner: Signer<'info>,
    pub sol_learn_account: Box<Account<'info, SolLearnInfo>>,
    /// CHECK:
    #[account(
        mut,
        seeds = [b"miner", miner.key().as_ref(), sol_learn_account.key().as_ref()], 
        bump = miner_account.bump,
    )]
    pub miner_account: Account<'info, MinerInfo>,
    #[account(
        mut, 
        realloc = 8 + 1 + 4 + miners_of_model.data.len() + 32,
        realloc::payer = miner,
        realloc::zero = false,
        seeds = [b"models", sol_learn_account.key().as_ref(), miner_account.model.key().as_ref()], 
        bump = miners_of_model.bump
    )]
    pub miners_of_model: Box<Account<'info, MinersOfModel>>,
    pub models: Box<Account<'info, Models>>,
    pub system_program: Program<'info, System>,
    pub sysvar_clock: Sysvar<'info, Clock>,
}

// Contract info
#[account]
pub struct SolLearnInfo {
    pub admin: Pubkey,
    pub token: Pubkey,
    pub total_miner: u64,
    pub total_models: u64,
    pub total_infer: u64,
    pub miner_minimum_stake: u64,
    pub unstake_delay_time: u64,
    pub reward_per_epoch: u64,
    pub min_fee_to_use: u64,
    pub last_epoch: u64,
    pub epoch_duration: u64,
    pub last_time: u64,

    pub inference_number: u64,
    pub assignment_number: u64,
    pub l2_owner: Pubkey,
    pub treasury: Pubkey,
    pub fee_l2_percentage: u16,
    pub fee_treasury_percentage: u16,
    pub fee_ratio_miner_validator: u16,
    pub submit_duration: u64,
    pub commit_duration: u64,
    pub reveal_duration: u64,
    pub penalty_duration: u64,
    pub miner_requirement: u8,
    pub blocks_per_epoch: u64,
    pub last_block: u64,
    pub fine_percentage: u16,
    pub dao_token_reward: u64,
    pub dao_token_percentage: DAOTokenPercentage,
}

impl SolLearnInfo {
    pub const LEN: usize = 32 + 32 + 8 * 10
            + 8 * 11 + 1 + 2 * 10 + 32 * 2;
}

#[account]
pub struct MinerInfo {
    pub bump: u8,
    pub miner: Pubkey,
    pub model: Pubkey,
    pub model_index: u64, // plus one already to make sure > 0
    pub stake_amount: u64,
    pub last_epoch: u64,
    pub active_time: u64,
    pub is_active: bool,
    pub unstaking_time: u64,
    pub reward: u64,
}

impl MinerInfo {
    pub const LEN: usize = 1 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 8 + 8;
}

#[account]
pub struct VaultAccount {
    pub bump: u8,  // 1 byte
}

impl VaultAccount {
    pub const LEN: usize = 8;
}

#[account]
pub struct Models {
    // pub bump: u8, 
    pub data: Vec<u8>,
    // pub minimum_fee: u64,
    // pub tier: u32,
}

impl Models {
    pub const LEN: usize = 1 + 4 + 8 + 8;
}

#[account]
pub struct MinersOfModel {
    pub bump: u8, 
    pub data: Vec<u8>,
}

impl MinersOfModel {
    pub const LEN: usize = 1 + 4;
}

#[account]
pub struct JoingMintingFlag {
    pub bump: u8,
    pub miner: Pubkey,
    pub model: Pubkey,
    pub stake_amount: u64,
    pub last_time: u64,
    pub active_time: u64,
}


// EVENTS
#[event]
pub struct MinerRegistration {
    pub miner: Pubkey,
    pub stake_amount: u64,
    pub model_address: Pubkey,
}

#[event]
pub struct MinerJoin {
    pub miner: Pubkey,
}

#[event]
pub struct MinerReJoin {
    pub miner: Pubkey,
}

#[event]
pub struct MinerTopup {
    pub miner: Pubkey,
    pub amount: u64,
}
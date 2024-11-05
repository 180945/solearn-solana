pub mod errors;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer};
use state::*;
use errors::*;

declare_id!("7MHr6ZPGTWZkRk6m52GfEWoMxSV7EoDjYyoXAYf3MBwS");

#[program]
pub mod solearn {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, min_stake: u64) -> Result<()> {
        msg!("Instruction: Initialize");

        let sol_learn_account = &mut ctx.accounts.sol_learn_account;

        sol_learn_account.admin = ctx.accounts.admin.key();
        sol_learn_account.token = ctx.accounts.staking_token.key();
        sol_learn_account.total_miner = 0;
        sol_learn_account.total_models = 0;
        sol_learn_account.total_infer = 0;
        sol_learn_account.miner_min_stake = min_stake;

        // vault account 
        ctx.accounts.vault_wallet_owner.bump = ctx.bumps.vault_wallet_owner;
        msg!("vault PDA bump seed: {}", ctx.bumps.vault_wallet_owner);

        // models
        ctx.accounts.models.bump = ctx.bumps.models;
        msg!("models PDA bump seed: {}", ctx.bumps.models);

        Ok(())
    }

    pub fn add_model(ctx: Context<AddModel>, model: Pubkey) -> Result<()> {
        msg!("Instruction: Add model");

        let models = &mut ctx.accounts.models;
        models.data.extend_from_slice(model.as_ref());
        ctx.accounts.sol_learn_account.total_models += 1;

        Ok(())
    }

    pub fn miner_register(ctx: Context<MinerRegister>, stake_amount: u64) -> Result<()> {
        msg!("Instruction: Miner register");

        // let user_info = &mut ctx.accounts.user_info;
        // let clock = Clock::get()?;

        // if user_info.amount > 0 {
        //     let reward = (clock.slot - user_info.deposit_slot) - user_info.reward_debt;

        //     let cpi_accounts = MintTo {
        //         mint: ctx.accounts.staking_token.to_account_info(),
        //         to: ctx.accounts.user_staking_wallet.to_account_info(),
        //         authority: ctx.accounts.admin.to_account_info(),
        //     };
        //     let cpi_program = ctx.accounts.token_program.to_account_info();
        //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        //     token::mint_to(cpi_ctx, reward)?;
        // }

        // let cpi_accounts = Transfer {
        //     from: ctx.accounts.user_staking_wallet.to_account_info(),
        //     to: ctx.accounts.admin_staking_wallet.to_account_info(),
        //     authority: ctx.accounts.user.to_account_info(),
        // };
        // let cpi_program = ctx.accounts.token_program.to_account_info();
        // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        // token::transfer(cpi_ctx, amount)?;

        // user_info.amount += amount;
        // user_info.deposit_slot = clock.slot;
        // user_info.reward_debt = 0;


        if ctx.accounts.sol_learn_account.miner_min_stake > stake_amount {
            return Err(SolLearnError::MustGreatThanMinStake.into())
        }

        // set miner info 
        let miner_info = &mut ctx.accounts.miner_info;
        miner_info.stake_amount = stake_amount;
        miner_info.last_epoch = ctx.accounts.sysvar_clock.epoch;

        // function registerMiner(
        //     uint16 tier,
        //     uint256 wEAIAmt
        // ) external whenNotPaused {
        //     _updateEpoch();
    
        //     if (tier == 0 || tier > maximumTier) revert InvalidTier();
        //     if (wEAIAmt < minerMinimumStake) revert StakeTooLow();
    
        //     Worker storage miner = miners[msg.sender];
        //     if (miner.tier != 0) revert AlreadyRegistered();
    
        //     miner.stake = wEAIAmt;
        //     miner.tier = tier;
    
        //     address modelAddress = modelAddresses.values[
        //         randomizer.randomUint256() % modelAddresses.size()
        //     ];
        //     miner.modelAddress = modelAddress;
        //     TransferHelper.safeTransferFrom(
        //         wEAI,
        //         msg.sender,
        //         address(this),
        //         wEAIAmt
        //     );
    
        //     emit MinerRegistration(msg.sender, tier, wEAIAmt);
        // }

        

        Ok(())
    }

    // pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
    //     msg!("Instruction: Unstake");

    //     let user_info = &mut ctx.accounts.user_info;
    //     let clock = Clock::get()?;

    //     let reward = (clock.slot - user_info.deposit_slot) - user_info.reward_debt;

    //     let cpi_accounts = MintTo {
    //         mint: ctx.accounts.staking_token.to_account_info(),
    //         to: ctx.accounts.user_staking_wallet.to_account_info(),
    //         authority: ctx.accounts.admin.to_account_info(),
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::mint_to(cpi_ctx, reward)?;

    //     let cpi_accounts = Transfer {
    //         from: ctx.accounts.admin_staking_wallet.to_account_info(),
    //         to: ctx.accounts.user_staking_wallet.to_account_info(),
    //         authority: ctx.accounts.admin.to_account_info(),
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::transfer(cpi_ctx, user_info.amount)?;

    //     user_info.amount = 0;
    //     user_info.deposit_slot = 0;
    //     user_info.reward_debt = 0;

    //     Ok(())
    // }

    // pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    //     msg!("Instruction: Claim Reward");

    //     let user_info = &mut ctx.accounts.user_info;
    //     let clock = Clock::get()?;

    //     let reward = (clock.slot - user_info.deposit_slot) - user_info.reward_debt;

    //     let cpi_accounts = MintTo {
    //         mint: ctx.accounts.staking_token.to_account_info(),
    //         to: ctx.accounts.user_staking_wallet.to_account_info(),
    //         authority: ctx.accounts.admin.to_account_info(),
    //     };
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    //     token::mint_to(cpi_ctx, reward)?;

    //     user_info.reward_debt += reward;

    //     Ok(())
    // }


}

fn random_number(clk: &Clock, range: u64) -> u64 {
    if range == 0 {
        return 0;
    }
    let leader_schedule_epoch = clk.leader_schedule_epoch;
    // let most_recent = array_ref![recent_blockhash_data, 0, 16];
    // u128::from_le_bytes(*most_recent)
    leader_schedule_epoch % range
}

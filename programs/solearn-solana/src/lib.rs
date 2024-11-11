pub mod errors;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
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

        if ctx.accounts.sol_learn_account.miner_min_stake > stake_amount {
            return Err(SolLearnError::MustGreatThanMinStake.into())
        }

        // set miner info 
        let miner_info = &mut ctx.accounts.miner_info;
        miner_info.stake_amount = stake_amount;
        miner_info.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;

        if ctx.accounts.models.data.len() == 0 {
            return Err(SolLearnError::NoModelRegistered.into())
        }

        // get random value 
        let model_index = random_number(&ctx.accounts.sysvar_clock, (ctx.accounts.models.data.len() / 32) as u64);
        let model: Pubkey = ctx.accounts.models.data[model_index as usize * 32..(model_index + 1) as usize * 32].try_into().expect("Invalid length");
        miner_info.model = model;
        ctx.accounts.sol_learn_account.total_miner += 1;

        // this used for unstaking 
        // let decimals = ctx.accounts.staking_token.decimals;
        // let solean_key = ctx.accounts.sol_learn_account.key().clone();
        // let seeds = &[
        //     &b"vault"[..], solean_key.as_ref()
        // ];

        // let signer_seeds = &[&seeds[..]];

        // // transfer token to contract
        // let cpi_accounts = TransferChecked {
        //     from: ctx.accounts.miner_staking_wallet.to_account_info(),
        //     to: ctx.accounts.vault_staking_wallet.to_account_info(),
        //     authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
        //     mint: ctx.accounts.staking_token.clone().to_account_info()
        // };

        // let ctx_transfer_token = CpiContext::new_with_signer(
        //     ctx.accounts.token_program.to_account_info(),
        //     cpi_accounts,
        //     signer_seeds
        // );

        // transfer_checked(ctx_transfer_token, stake_amount, decimals)?;

        let cpi_accounts = Transfer {
            from: ctx.accounts.miner_staking_wallet.to_account_info(),
            to: ctx.accounts.vault_staking_wallet.to_account_info(),
            authority: ctx.accounts.miner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, stake_amount)?;
        
        // emit event
        emit!(MinerRegistration {
            miner: *ctx.accounts.miner.key,
            stake_amount,
            model_address: model,
        });

        Ok(())
    }

    pub fn join_for_minting(ctx: Context<JoinForMinting>) -> Result<()> {
        msg!("Instruction: Join For Minting");

        // Assuming _updateEpoch() is a function that updates the epoch based on the current clock
        // _update_epoch(&ctx.accounts.sysvar_clock)?;

        if ctx.accounts.sol_learn_account.miner_min_stake > ctx.accounts.miner_account.stake_amount {
            return Err(SolLearnError::MustGreatThanMinStake.into())
        }

        // get active time
        if ctx.accounts.miner_account.active_time > (ctx.accounts.sysvar_clock.unix_timestamp as u64) {
            return Err(SolLearnError::NotAcitveYet.into())
        }

        if ctx.accounts.miner_account.model_index > 0 {
            return Err(SolLearnError::Joined.into())
        }

        // insert model address
        let addresses_of_model = &mut ctx.accounts.addresses_of_model;
        addresses_of_model.data.extend_from_slice(ctx.accounts.miner.key().as_ref());

        // update miner join time
        ctx.accounts.miner_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;
        ctx.accounts.miner_account.is_active = true;
        ctx.accounts.miner_account.model_index = (addresses_of_model.data.len() / 32 + 1) as u64;
        emit!(MinerJoin {
            miner: *ctx.accounts.miner.key,
        });

        Ok(())
    }

    // this handle case when miner slashed and wanna join mining again
    pub fn rejoin_mining(ctx: Context<ReJoinForMinting>) -> Result<()> {
        msg!("Instruction: ReJoin For Minting");

        // Assuming _updateEpoch() is a function that updates the epoch based on the current clock
        // _update_epoch(&ctx.accounts.sysvar_clock)?;

        if ctx.accounts.miner_account.is_active {
            return Err(SolLearnError::Activated.into())
        }

        if ctx.accounts.miner_account.model_index > 0 {
            return Err(SolLearnError::MustJoinMintingFirst.into())
        }

        // check is enough token to rejoin mining
        if ctx.accounts.sol_learn_account.miner_min_stake > ctx.accounts.miner_account.stake_amount {
            return Err(SolLearnError::MustGreatThanMinStake.into())
        }

        // update miner join time
        ctx.accounts.miner_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;
        ctx.accounts.miner_account.is_active = true;

        emit!(MinerReJoin {
            miner: *ctx.accounts.miner.key,
        });

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

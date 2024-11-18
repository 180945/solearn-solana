pub mod errors;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TransferChecked, transfer_checked};
use state::*;
use errors::*;

declare_id!("7MHr6ZPGTWZkRk6m52GfEWoMxSV7EoDjYyoXAYf3MBwS");

#[program]
pub mod solearn {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, min_stake: u64, reward_per_epoch: u64, epoch_duration: u64) -> Result<()> {
        msg!("Instruction: Initialize");

        let sol_learn_account = &mut ctx.accounts.sol_learn_account;

        sol_learn_account.admin = ctx.accounts.admin.key();
        sol_learn_account.token = ctx.accounts.staking_token.key();
        sol_learn_account.total_miner = 0;
        sol_learn_account.total_models = 0;
        sol_learn_account.total_infer = 0;
        sol_learn_account.miner_min_stake = min_stake;
        sol_learn_account.reward_per_epoch = reward_per_epoch;
        sol_learn_account.epoch_duration = epoch_duration;
        sol_learn_account.last_epoch = 0;
        sol_learn_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;

        // vault account 
        ctx.accounts.vault_wallet_owner.bump = ctx.bumps.vault_wallet_owner;
        msg!("vault PDA bump seed: {}", ctx.bumps.vault_wallet_owner);

        // models
        ctx.accounts.models.bump = ctx.bumps.models;
        msg!("models PDA bump seed: {}", ctx.bumps.models);

        Ok(())
    }

    pub fn miner_register(ctx: Context<MinerRegister>, stake_amount: u64) -> Result<()> {
        msg!("Instruction: Miner register");

        if ctx.accounts.sol_learn_account.miner_min_stake > stake_amount {
            return Err(SolLearnError::MustGreatThanMinStake.into())
        }

        // set miner info 
        let miner_account = &mut ctx.accounts.miner_account;
        miner_account.stake_amount = stake_amount;

        if ctx.accounts.models.data.len() == 0 {
            return Err(SolLearnError::NoModelRegistered.into())
        }

        // get random value 
        let model_index = random_number(&ctx.accounts.sysvar_clock, (ctx.accounts.models.data.len() / 32) as u64);
        let model: Pubkey = ctx.accounts.models.data[model_index as usize * 32..(model_index + 1) as usize * 32].try_into().expect("Invalid length");
        miner_account.model = model;
        ctx.accounts.sol_learn_account.total_miner += 1;

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

        // update epoch section
        let n = ((ctx.accounts.sysvar_clock.unix_timestamp as u64) - ctx.accounts.sol_learn_account.last_time) / ctx.accounts.sol_learn_account.epoch_duration;
        if n > 0 {
            ctx.accounts.sol_learn_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;
            ctx.accounts.sol_learn_account.last_epoch += n;
        }

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

        if ctx.accounts.miner_account.is_active {
            return Err(SolLearnError::Activated.into())
        }

        // insert model address
        let miners_of_model = &mut ctx.accounts.miners_of_model;
        miners_of_model.data.extend_from_slice(ctx.accounts.miner.key().as_ref());

        // update miner join epoch time
        ctx.accounts.miner_account.last_epoch = ctx.accounts.sol_learn_account.last_epoch;        
        ctx.accounts.miner_account.is_active = true;
        ctx.accounts.miner_account.model_index = (miners_of_model.data.len() / 32 + 1) as u64;

        // handle case miner cancle unstaking
        if ctx.accounts.miner_account.unstaking_time > 0 {
            ctx.accounts.miner_account.unstaking_time = 0;
        }

        emit!(MinerJoin {
            miner: *ctx.accounts.miner.key,
        });

        Ok(())
    }
    
    // topup
    pub fn topup(ctx: Context<Topup>, topup_amount: u64) -> Result<()> {
        msg!("Instruction: Top up staking amount");

        let miner_info = &mut ctx.accounts.miner_info;
        miner_info.stake_amount += topup_amount;

        let cpi_accounts = Transfer {
            from: ctx.accounts.miner_staking_wallet.to_account_info(),
            to: ctx.accounts.vault_staking_wallet.to_account_info(),
            authority: ctx.accounts.miner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, topup_amount)?;
        
        // emit event
        emit!(MinerTopup {
            miner: *ctx.accounts.miner.key,
            amount: topup_amount,
        });

        Ok(())
    } 

    // unregister_miner
    pub fn miner_unstaking(ctx: Context<MinerUnStaking>) -> Result<()> {
        msg!("Instruction: Miner unregister");

        // update epoch section
        let n = ((ctx.accounts.sysvar_clock.unix_timestamp as u64) - ctx.accounts.sol_learn_account.last_time) / ctx.accounts.sol_learn_account.epoch_duration;
        if n > 0 {
            ctx.accounts.sol_learn_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;
            ctx.accounts.sol_learn_account.last_epoch += n;
        }

        if ctx.accounts.miner_account.model_index != 0 {
            return Err(SolLearnError::MinerNotRegistered.into())
        }

        if ctx.accounts.miner_account.unstaking_time != 0 {
            return Err(SolLearnError::Unstaked.into())
        }

        // update account unstaking time
        ctx.accounts.miner_account.unstaking_time = (ctx.accounts.sysvar_clock.unix_timestamp as u64) + ctx.accounts.sol_learn_account.unstake_delay_time;
        // update mining reward
        if ctx.accounts.miner_account.is_active {
            ctx.accounts.miner_account.is_active = false;
            ctx.accounts.miner_account.reward += (ctx.accounts.sol_learn_account.last_epoch - ctx.accounts.miner_account.last_epoch) * ctx.accounts.sol_learn_account.reward_per_epoch;
        }

        // remove from MinersOfModel
        let miner_key = ctx.accounts.miner.key();
        let mut data = ctx.accounts.miners_of_model.data.clone();
        
        // Find the index of the miner's key in the data
        if let Some(index) = data.chunks(32).position(|chunk| chunk == miner_key.as_ref()) {
            // Remove the miner's key from the data
            data.drain(index * 32..(index + 1) * 32);
            
            // Update the account data
            ctx.accounts.miners_of_model.data = data;
        } else {
            return Err(SolLearnError::MinerNotRegistered.into());
        }

        Ok(())
    }

    // claim unstaking amount 
    pub fn miner_claim_unstaked(ctx: Context<MinerClaim>) -> Result<()> {
        
        if ctx.accounts.miner_account.is_active {
            return Err(SolLearnError::Activated.into())
        }

        if ctx.accounts.miner_account.unstaking_time == 0 || ctx.accounts.miner_account.unstaking_time > (ctx.accounts.sysvar_clock.unix_timestamp as u64) {
            return Err(SolLearnError::CanNotClaim.into())
        }

        let unstake_amount = ctx.accounts.miner_account.stake_amount;
        if unstake_amount == 0 {
            return Err(SolLearnError::NothingToClaim.into())
        }
        ctx.accounts.miner_account.stake_amount = 0;
        ctx.accounts.miner_account.unstaking_time = 0;

        // this used for unstaking 
        let decimals = ctx.accounts.staking_token.decimals;
        let solean_key = ctx.accounts.sol_learn_account.key().clone();
        let seeds = &[
            "vault".as_bytes(), solean_key.as_ref(), &[ctx.accounts.vault_wallet_owner_pda.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        // transfer token to contract
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.miner_staking_wallet.to_account_info(),
            to: ctx.accounts.vault_staking_wallet.to_account_info(),
            authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
            mint: ctx.accounts.staking_token.clone().to_account_info()
        };

        let ctx_transfer_token = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds
        );

        transfer_checked(ctx_transfer_token, unstake_amount, decimals)?;


        Ok(())
    }

    // claim reward
    pub fn miner_claim_reward(ctx: Context<MinerClaimReward>) -> Result<()> {
        
        // update epoch section
        let n = ((ctx.accounts.sysvar_clock.unix_timestamp as u64) - ctx.accounts.sol_learn_account.last_time) / ctx.accounts.sol_learn_account.epoch_duration;
        if n > 0 {
            ctx.accounts.sol_learn_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;
            ctx.accounts.sol_learn_account.last_epoch += n;
        }

        // udpate latest reward
        let reward = ctx.accounts.miner_account.reward + (ctx.accounts.sol_learn_account.last_epoch - ctx.accounts.miner_account.last_epoch) * ctx.accounts.sol_learn_account.reward_per_epoch;
        if reward == 0 {
            return Err(SolLearnError::NothingToClaim.into())
        }

        ctx.accounts.miner_account.last_epoch = ctx.accounts.sol_learn_account.last_epoch;

        // this used for unstaking 
        let decimals = ctx.accounts.staking_token.decimals;
        let solean_key = ctx.accounts.sol_learn_account.key().clone();
        let seeds = &[
            "vault".as_bytes(), solean_key.as_ref(), &[ctx.accounts.vault_wallet_owner_pda.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        // transfer token to contract
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.miner_staking_wallet.to_account_info(),
            to: ctx.accounts.vault_staking_wallet.to_account_info(),
            authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
            mint: ctx.accounts.staking_token.clone().to_account_info()
        };

        let ctx_transfer_token = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds
        );

        transfer_checked(ctx_transfer_token, reward, decimals)?;

        Ok(())
    }

    // ADMIN section
    // todos: 

    // add model
    pub fn add_model(ctx: Context<AddModel>, model: Pubkey) -> Result<()> {
        msg!("Instruction: Add model");

        let models = &mut ctx.accounts.models;
        models.data.extend_from_slice(model.as_ref());
        ctx.accounts.sol_learn_account.total_models += 1;

        Ok(())
    }

    // remove model
    pub fn remove_model(ctx: Context<RemoveModel>, model: Pubkey) -> Result<()> {
        msg!("Instruction: Add model");

        let mut data = ctx.accounts.models.data.clone();
        // Find the index of the miner's key in the data
        if let Some(index) = data.chunks(32).position(|chunk| chunk == model.as_ref()) {
            // Remove the miner's key from the data
            data.drain(index * 32..(index + 1) * 32);
            
            // Update the models data
            ctx.accounts.models.data = data;
        } else {
            return Err(SolLearnError::ModelNotExist.into());
        }

        ctx.accounts.sol_learn_account.total_models -= 1;
        

        Ok(())
    }

    
    // epoch update
    // set fine percentage
    // setPenaltyDuration
    // setMinFeeToUse
    // setNewRewardInEpoch

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

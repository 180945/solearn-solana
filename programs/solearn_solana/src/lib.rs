pub mod errors;
pub mod state;
pub mod state_inf;
mod utils;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hash;
use anchor_spl::token::{self, transfer_checked, Transfer, TransferChecked};
use errors::*;
use state::*;
use state_inf::*;
use utils::*;

declare_id!("8fXqHtGHRwr7Mdif7HJhsV66qwSWLnozjabo2uEHPFZ1");

#[program]
pub mod solearn {
    use anchor_lang::accounts;

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        reward_per_epoch: u64,
        epoch_duration: u64,
        min_stake: u64,
        treasury_address: Pubkey,
        fee_l2_percentage: u16,
        fee_treasury_percentage: u16,
        fee_ratio_miner_validator: u16,
        submit_duration: u64,
        commit_duration: u64,
        reveal_duration: u64,
        penalty_duration: u64,
        miner_requirement: u8,
        blocks_per_epoch: u64, // todo: consider to remove this value, use epoch_duration instead
        fine_percentage: u16,
        dao_token_reward: u64,
        miner_percentage: u16,
        user_percentage: u16,
        referrer_percentage: u16,
        referee_percentage: u16,
        l2_owner_percentage: u16,
        unstake_delay_time: u64,
    ) -> Result<()> {
        msg!("Instruction: Initialize");

        let sol_learn_account = &mut ctx.accounts.sol_learn_account;

        sol_learn_account.admin = ctx.accounts.admin.key();
        sol_learn_account.token = ctx.accounts.staking_token.key();
        sol_learn_account.total_miner = 0;
        sol_learn_account.total_models = 0;
        sol_learn_account.total_infer = 0;
        sol_learn_account.miner_minimum_stake = min_stake;
        sol_learn_account.reward_per_epoch = reward_per_epoch;
        sol_learn_account.epoch_duration = epoch_duration;
        sol_learn_account.last_epoch = 0;
        sol_learn_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;
        sol_learn_account.inference_number = 0;
        sol_learn_account.assignment_number = 0;
        sol_learn_account.l2_owner = ctx.accounts.admin.key();
        sol_learn_account.treasury = treasury_address;
        sol_learn_account.last_block = Clock::get()?.slot;
        sol_learn_account.fee_l2_percentage = fee_l2_percentage;
        sol_learn_account.fee_treasury_percentage = fee_treasury_percentage;
        sol_learn_account.fee_ratio_miner_validator = fee_ratio_miner_validator;
        sol_learn_account.submit_duration = submit_duration;
        sol_learn_account.commit_duration = commit_duration;
        sol_learn_account.reveal_duration = reveal_duration;
        sol_learn_account.penalty_duration = penalty_duration;
        sol_learn_account.miner_requirement = miner_requirement;
        sol_learn_account.blocks_per_epoch = blocks_per_epoch;
        sol_learn_account.fine_percentage = fine_percentage;
        sol_learn_account.dao_token_reward = dao_token_reward;
        sol_learn_account.dao_token_percentage.miner_percentage = miner_percentage;
        sol_learn_account.dao_token_percentage.user_percentage = user_percentage;
        sol_learn_account.dao_token_percentage.referrer_percentage = referrer_percentage;
        sol_learn_account.dao_token_percentage.referee_percentage = referee_percentage;
        sol_learn_account.dao_token_percentage.l2_owner_percentage = l2_owner_percentage;
        sol_learn_account.unstake_delay_time = unstake_delay_time;

        // vault account
        ctx.accounts.vault_wallet_owner_pda.bump = ctx.bumps.vault_wallet_owner_pda;
        msg!("vault PDA bump seed: {}", ctx.bumps.vault_wallet_owner_pda);

        

        Ok(())
    }

    pub fn initialize2(ctx: Context<InitializeExtra>) -> Result<()> {
        msg!("Instruction: Initialize2");

        let t = &mut ctx.accounts.tasks;
        t.values = vec![];
        t.bump = ctx.bumps.tasks;

        msg!("Tasks bump seed: {}", ctx.bumps.tasks);

        Ok(())
    }

    pub fn miner_register(ctx: Context<MinerRegister>, stake_amount: u64) -> Result<()> {
        msg!("Instruction: Miner register");

        if ctx.accounts.sol_learn_account.miner_minimum_stake > stake_amount {
            return Err(SolLearnError::MustGreatThanMinStake.into());
        }

        // set miner info
        let miner_account = &mut ctx.accounts.miner_account;
        miner_account.stake_amount = stake_amount;
        miner_account.bump = ctx.bumps.miner_account;

        if ctx.accounts.models.data.len() == 0 {
            return Err(SolLearnError::NoModelRegistered.into());
        }

        if ctx.accounts.sol_learn_account.token != ctx.accounts.staking_token.key() {
            return Err(SolLearnError::InvalidToken.into());
        }

        // get random value
        let model_index = random_number(
            &&Clock::get()?,
            0,
            (ctx.accounts.models.data.len() / 32) as u64,
        );
        let model: Pubkey = ctx.accounts.models.data
            [model_index as usize * 32..(model_index + 1) as usize * 32]
            .try_into()
            .expect("Invalid length");
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
        let n = ((ctx.accounts.sysvar_clock.unix_timestamp as u64)
            - ctx.accounts.sol_learn_account.last_time)
            / ctx.accounts.sol_learn_account.epoch_duration;
        if n > 0 {
            ctx.accounts.sol_learn_account.last_time =
                ctx.accounts.sysvar_clock.unix_timestamp as u64;
            ctx.accounts.sol_learn_account.last_epoch += n;
        }

        if ctx.accounts.sol_learn_account.miner_minimum_stake > ctx.accounts.miner_account.stake_amount
        {
            return Err(SolLearnError::MustGreatThanMinStake.into());
        }

        // get active time
        if ctx.accounts.miner_account.active_time
            > (ctx.accounts.sysvar_clock.unix_timestamp as u64)
        {
            return Err(SolLearnError::NotAcitveYet.into());
        }

        if ctx.accounts.miner_account.is_active {
            return Err(SolLearnError::Joined.into());
        }

        // insert model address
        let miners_of_model: &mut Account<'_, MinersOfModel> = &mut ctx.accounts.miners_of_model;
        miners_of_model
            .data
            .extend_from_slice(ctx.accounts.miner.key().as_ref());

        // update miner join epoch time
        ctx.accounts.miner_account.last_epoch = ctx.accounts.sol_learn_account.last_epoch;
        ctx.accounts.miner_account.is_active = true;

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

        if ctx.accounts.sol_learn_account.token != ctx.accounts.staking_token.key() {
            return Err(SolLearnError::InvalidToken.into());
        }

        let miner_account = &mut ctx.accounts.miner_account;
        miner_account.stake_amount += topup_amount;

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
    pub fn miner_unstake(ctx: Context<MinerUnStaking>, model_index: u64) -> Result<()> {
        msg!("Instruction: Miner unstake");

        // update epoch section
        let n = ((ctx.accounts.sysvar_clock.unix_timestamp as u64)
            - ctx.accounts.sol_learn_account.last_time)
            / ctx.accounts.sol_learn_account.epoch_duration;
        if n > 0 {
            ctx.accounts.sol_learn_account.last_time =
                ctx.accounts.sysvar_clock.unix_timestamp as u64;
            ctx.accounts.sol_learn_account.last_epoch += n;
        }

        if ctx.accounts.miner_account.stake_amount == 0 {
            return Err(SolLearnError::StakingZeroValue.into());
        }

        if ctx.accounts.miner_account.unstaking_time != 0 {
            return Err(SolLearnError::Unstaking.into());
        }

        // update account unstaking time
        ctx.accounts.miner_account.unstaking_time = (ctx.accounts.sysvar_clock.unix_timestamp
            as u64)
            + ctx.accounts.sol_learn_account.unstake_delay_time;

        if ctx.accounts.miner_account.is_active {
            ctx.accounts.miner_account.reward += (ctx.accounts.sol_learn_account.last_epoch
                - ctx.accounts.miner_account.last_epoch)
                * ctx.accounts.sol_learn_account.reward_per_epoch;

            // remove from MinersOfModel
            let miner_key = ctx.accounts.miner.key();
            let mut data = ctx.accounts.miners_of_model.data.clone();

            if (data.len() as u64) < 32 * (model_index + 1) {
                return Err(SolLearnError::InvalidModelIndex.into());
            }

            let miner_key_extracted: Vec<u8> = data[(model_index as usize) * 32..(model_index as usize + 1) * 32].to_vec();
            if miner_key_extracted == miner_key.try_to_vec()? {
                data.drain((model_index as usize) * 32..(model_index as usize + 1) * 32);
             
                // Update the account data
                ctx.accounts.miners_of_model.data = data;
                ctx.accounts.miner_account.is_active = false;
            } else {
                return Err(SolLearnError::MinerNotRegistered.into());
            }
            
        }

        Ok(())
    }

    // claim unstaking amount
    pub fn miner_claim_unstaked(ctx: Context<MinerClaim>) -> Result<()> {
        if ctx.accounts.sol_learn_account.token != ctx.accounts.staking_token.key() {
            return Err(SolLearnError::InvalidToken.into());
        }

        if ctx.accounts.miner_account.is_active {
            return Err(SolLearnError::Activated.into());
        }

        if ctx.accounts.miner_account.unstaking_time == 0
            || ctx.accounts.miner_account.unstaking_time
                > (ctx.accounts.sysvar_clock.unix_timestamp as u64)
        {
            return Err(SolLearnError::CanNotClaim.into());
        }

        let unstake_amount = ctx.accounts.miner_account.stake_amount;
        if unstake_amount == 0 {
            return Err(SolLearnError::NothingToClaim.into());
        }
        ctx.accounts.miner_account.stake_amount = 0;
        ctx.accounts.miner_account.unstaking_time = 0;

        // this used for unstaking
        let decimals = ctx.accounts.staking_token.decimals;
        let solearn_key = ctx.accounts.sol_learn_account.key().clone();
        let seeds = &[
            "vault".as_bytes(),
            solearn_key.as_ref(),
            &[ctx.accounts.vault_wallet_owner_pda.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        // transfer token to contract
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.vault_staking_wallet.to_account_info(),
            to: ctx.accounts.miner_staking_wallet.to_account_info(),
            authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
            mint: ctx.accounts.staking_token.clone().to_account_info(),
        };

        let ctx_transfer_token = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer_checked(ctx_transfer_token, unstake_amount, decimals)?;

        Ok(())
    }

    // claim reward
    pub fn miner_claim_reward(ctx: Context<MinerClaimReward>) -> Result<()> {
        if ctx.accounts.sol_learn_account.token != ctx.accounts.staking_token.key() {
            return Err(SolLearnError::InvalidToken.into());
        }

        // update epoch section
        let n = ((ctx.accounts.sysvar_clock.unix_timestamp as u64)
            - ctx.accounts.sol_learn_account.last_time)
            / ctx.accounts.sol_learn_account.epoch_duration;
        if n > 0 {
            ctx.accounts.sol_learn_account.last_time =
                ctx.accounts.sysvar_clock.unix_timestamp as u64;
            ctx.accounts.sol_learn_account.last_epoch += n;
        }

        let mut reward = 0;
        if ctx.accounts.miner_account.is_active {
            // udpate latest reward
            reward = ctx.accounts.miner_account.reward
                + (ctx.accounts.sol_learn_account.last_epoch
                    - ctx.accounts.miner_account.last_epoch)
                    * ctx.accounts.sol_learn_account.reward_per_epoch;
            if reward == 0 {
                return Err(SolLearnError::NothingToClaim.into());
            }
        }
        ctx.accounts.miner_account.reward = 0;
        ctx.accounts.miner_account.last_epoch = ctx.accounts.sol_learn_account.last_epoch;

        // this used for unstaking
        let decimals = ctx.accounts.staking_token.decimals;
        let solean_key = ctx.accounts.sol_learn_account.key().clone();
        let seeds = &[
            "vault".as_bytes(),
            solean_key.as_ref(),
            &[ctx.accounts.vault_wallet_owner_pda.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        // transfer token to contract
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.miner_staking_wallet.to_account_info(),
            to: ctx.accounts.vault_staking_wallet.to_account_info(),
            authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
            mint: ctx.accounts.staking_token.clone().to_account_info(),
        };

        let ctx_transfer_token = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
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
        ctx.accounts.miners_of_model.bump = ctx.bumps.miners_of_model;
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

    pub fn set_miner_min_stake(ctx: Context<UpdateParamsVld>, data: u64) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        acc.miner_minimum_stake = data.into();
        Ok(())
    }

    pub fn next_inference_id(ctx: Context<ReadStateVld>) -> Result<u64> {
        let acc = &ctx.accounts.sol_learn_account;
        Ok(acc.inference_number + 1)
    }

    pub fn next_assignment_id(ctx: Context<ReadStateVld>) -> Result<u64> {
        let acc = &ctx.accounts.sol_learn_account;
        Ok(acc.assignment_number + 1)
    }

    pub fn next_epoch_id(ctx: Context<ReadStateVld>) -> Result<u64> {
        let acc = &ctx.accounts.sol_learn_account;
        Ok(acc.last_epoch + 1)
    }

    pub fn get_task_count(ctx: Context<ReadTasksVld>) -> Result<u64> {
        let t = &ctx.accounts.tasks;
        Ok((t.values.len() as u64) / 50)
    }

    pub fn get_assignment(ctx: Context<ReadAssignmentVld>, assignment_id: u64, field_name: String) -> Result<Vec<u8>> {
        let asgnmt = &ctx.accounts.assignment;
        if asgnmt.id != assignment_id {
            return Err(SolLearnError::Unauthorized.into());
        }

        match field_name.as_str() {
            "inference_id" => Ok(asgnmt.inference_id.to_le_bytes().to_vec()),
            "worker" => Ok(asgnmt.worker.to_bytes().to_vec()),
            "role" => Ok(vec![asgnmt.role]),
            "vote" => Ok(vec![asgnmt.vote]),
            "reveal_nonce" => Ok(asgnmt.reveal_nonce.to_le_bytes().to_vec()),
            "output" => Ok(asgnmt.output.to_vec()),
            "commitment" => Ok(asgnmt.commitment.to_vec()),
            "digest" => Ok(asgnmt.digest.to_vec()),
            _ => Err(SolLearnError::UnknownStructField.into()),
        }
    }

    pub fn update_epoch(ctx: Context<UpdateEpochVld>, epoch_id: u64) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;

        let slot_number = Clock::get()?.slot;
        let epoch_passed = (slot_number - acc.last_block) / acc.blocks_per_epoch;
        if epoch_passed > 0 {
            if epoch_id > acc.last_epoch {
                return Err(SolLearnError::InvalidEpochId.into());
            }

            acc.last_block += acc.blocks_per_epoch * epoch_passed;
            let reward_in_current_epoch =
                (acc.reward_per_epoch * acc.blocks_per_epoch) / BLOCK_PER_YEAR;

            let ms = &mut ctx.accounts.miner_reward;
            ms.total_miner = acc.total_miner;
            ms.epoch_reward = reward_in_current_epoch;
            acc.last_epoch += 1;
            Ok(())
        } else {
            return Err(SolLearnError::EpochRewardUpToDate.into());
        }
    }

    pub fn infer(
        ctx: Context<InferVld>,
        inference_id: u64,
        creator: Pubkey,
        input: Vec<u8>,
        _value: u64,
        model: Pubkey,
    ) -> Result<u64> {
        let acc = &mut ctx.accounts.sol_learn_account;
        let mdls = &mut ctx.accounts.models;
        let miners_of_model = &mut ctx.accounts.miners_of_model;
        // let referrer = &mut ctx.accounts.referrer;
        // referrer.bump = ctx.bumps.referrer;

        msg!("miner len {}", miners_of_model.data.len());
        // if model.tier == 0 {
        //     return Err(SolLearnError::Unauthorized.into());
        // }

        // find model
        for i in 0..mdls.data.len() / 32 {
            let b: [u8; 32] = mdls.data[i * 32..(i + 1) * 32].try_into().unwrap();
            let model_pubkey = Pubkey::new_from_array(b);
            if model_pubkey == model {
                break;
            }
            if i == mdls.data.len() / 32 - 1 {
                return Err(SolLearnError::ModelNotExist.into());
            }
        }

        // let b: [u8; 32] = model.data[0..32].try_into().unwrap();
        // let model_pubkey = Pubkey::new_from_array(b);

        let scoring_fee = validate_enough_fee_to_use(acc.min_fee_to_use, _value)?;
        // let from = ctx.accounts.signer.to_account_info();
        // let to = ctx.accounts.vault_wallet_owner_pda.to_account_info();
        // if **from.try_borrow_lamports()? < _value {
        //     return Err(SolLearnError::InsufficientFunds.into());
        // }
        // **from.try_borrow_mut_lamports()? -= _value;
        // **to.try_borrow_mut_lamports()? += _value;

        let value = _value - scoring_fee;

        acc.inference_number += 1;
        if inference_id != acc.inference_number {
            return Err(SolLearnError::WrongInferenceId.into());
        }
        let inference = &mut ctx.accounts.infs;

        let fee_l2 = (value * u64::from(acc.fee_l2_percentage)) / PERCENTAGE_DENOMINATOR;
        let fee_treasury =
            (value * u64::from(acc.fee_treasury_percentage)) / PERCENTAGE_DENOMINATOR;

        inference.id = inference_id;
        inference.input = input;
        inference.fee_l2 = fee_l2;
        inference.fee_treasury = fee_treasury;
        inference.value = value - fee_l2 - fee_treasury;
        inference.creator = creator;
        // inference.referrer = referrer.pubkey;
        inference.model_address = model;
        inference.bump = ctx.bumps.infs;

        let slot_number = Clock::get()?.slot;
        let expired_at = slot_number + acc.submit_duration;
        let commit_timeout = expired_at + acc.commit_duration;
        inference.submit_timeout = expired_at;
        inference.commit_timeout = commit_timeout;
        inference.reveal_timeout = commit_timeout + acc.reveal_duration;
        inference.status = 1;
        inference.assignments = vec![];

        let n = acc.miner_requirement;
        let mut selected_miners = Vec::with_capacity(n as usize);
        let tasks = &mut ctx.accounts.tasks;
        msg!("tasks len: {}", tasks.values.len());
        msg!("bump: {}", tasks.bump);


        for i in 0..n {
            let rand_uint = random_number(
                &&Clock::get()?,
                i.into(),
                (miners_of_model.data.len() / 32) as u64,
            );

            let miner_ind = (rand_uint as usize) % (miners_of_model.data.len() / 32);
            msg!("random number {} and miner #{} was chosen", rand_uint, miner_ind);

            let miner_bytes = miners_of_model.data.drain(miner_ind*32..(miner_ind+1)*32).collect::<Vec<u8>>();
            let miner = Pubkey::new_from_array(miner_bytes.try_into().unwrap());
            msg!("miner chosen: {:?}", miner);



            acc.assignment_number += 1;
            let assignment_id = acc.assignment_number;
            
            let mut data = vec![];
            data.extend_from_slice(&assignment_id.to_le_bytes());
            data.extend_from_slice(&inference_id.to_le_bytes());
            data.extend_from_slice(&miner.to_bytes());
            data.push(1);
            
            tasks.push_task(Task::new(FnType::CreateAssignment, data.clone()));
            msg!("push task: {:?}", data);

            selected_miners.push(miner);
            // assignments_by_miner[miner].insert(assignment_id);
            // assignments_by_inference[inference_id].insert(assignment_id);
        }

        for miner in selected_miners {
            miners_of_model.data.extend_from_slice(miner.to_bytes().as_ref());
        }
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.miner_staking_wallet.to_account_info(),
            to: ctx.accounts.vault_staking_wallet.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, _value)?;

        emit!(NewInference {
            inference_id,
            creator,
            model_address: model,
            value,
        });

        Ok(0)
    }

    pub fn create_assignment(ctx: Context<CreateAssignmentVld>, assignment_id: u64) -> Result<()> {
        let tasks = &mut ctx.accounts.tasks;
        let task;
        match tasks.pop_task() {
            Some(t) => task = t,
            None => return Err(SolLearnError::NoValidTask.into()),
        }
        if task.fn_type() != FnType::CreateAssignment {
            return Err(SolLearnError::NoValidTask.into());
        }
        let data = task.data();
        let mut assignment_id_bytes = [0u8; 8];
        assignment_id_bytes.copy_from_slice(&data[0..8]);
        let check_assignment_id = u64::from_le_bytes(assignment_id_bytes);
        if check_assignment_id != assignment_id {
            return Err(SolLearnError::WrongAssignmentId.into());
        }

        let mut inference_id_bytes = [0u8; 8];
        inference_id_bytes.copy_from_slice(&data[8..16]);
        let inference_id = u64::from_le_bytes(inference_id_bytes);
        let worker = Pubkey::try_from(&data[16..48]).unwrap();
        let role = data[48];

        let assignment = &mut ctx.accounts.assignment;
        assignment.inference_id = inference_id;
        assignment.worker = worker;
        assignment.role = role;
        assignment.bump = ctx.bumps.assignment;
        assignment.id = assignment_id;

        emit!(NewAssignment {
            assignment_id,
            inference_id,
            worker,
        });

        Ok(())
    }

    pub fn topup_infer(ctx: Context<UpdateInferVld>, inference_id: u64, value: u64) -> Result<()> {
        if value == 0 {
            return Err(SolLearnError::ZeroValue.into());
        }
        
        // let from = ctx.accounts.signer.to_account_info();
        // let to = ctx.accounts.vault_wallet_owner_pda.to_account_info();
        // if **from.try_borrow_lamports()? < value {
        //     return Err(SolLearnError::InsufficientFunds.into());
        // }
        // **from.try_borrow_mut_lamports()? -= value;
        // **to.try_borrow_mut_lamports()? += value;

        let inference = &mut ctx.accounts.infs;
        if inference_id != inference.id {
            return Err(SolLearnError::WrongInferenceId.into());
        }
        if inference.status != 1 {
            return Err(SolLearnError::InferMustBeSolvingState.into());
        }

        inference.value += value;

        let cpi_accounts = Transfer {
            from: ctx.accounts.miner_staking_wallet.to_account_info(),
            to: ctx.accounts.vault_staking_wallet.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, value)?;

        emit!(TopUpInfer {
            inference_id,
            creator: ctx.accounts.signer.key(),
            value,
        });

        Ok(())
    }

    pub fn seize_miner_role(ctx: Context<SeizeMinerRoleVld>, assignment_id: u64, inference_id: u64) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        let inference = &mut ctx.accounts.infs;
        let assignment = &mut ctx.accounts.assignment;

        if assignment_id != assignment.id {
            return Err(SolLearnError::Unauthorized.into());
        }

        // only_updated_epoch(acc)?;

        if assignment.worker != ctx.accounts.signer.key() {
            return Err(SolLearnError::Unauthorized.into());
        }

        if inference_id != assignment.inference_id {
            return Err(SolLearnError::Unauthorized.into());
        }

        if inference.processed_miner != Pubkey::default() {
            return Err(SolLearnError::InferenceSeized.into());
        }

        assignment.role = 2;
        inference.processed_miner = ctx.accounts.signer.key();

        emit!(MinerRoleSeized {
            assignment_id,
            inference_id: inference_id,
            sender: ctx.accounts.signer.key(),
        });

        Ok(())
    }

    pub fn submit_solution(
        ctx: Context<SeizeMinerRoleVld>,
        assignment_id: u64,
        inference_id: u64,
        data: Vec<u8>,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        let assignment = &mut ctx.accounts.assignment;
        let inference = &mut ctx.accounts.infs;

        let infer_id = assignment.inference_id;

        if assignment_id != assignment.id {
            return Err(SolLearnError::Unauthorized.into());
        }
        if inference.id != infer_id || inference.id != inference_id {
            return Err(SolLearnError::Unauthorized.into());
        }
        if ctx.accounts.signer.key() != assignment.worker {
            return Err(SolLearnError::Unauthorized.into());
        }
        if assignment.role != 2 {
            msg!("role: {}", assignment.role);
            return Err(SolLearnError::Unauthorized.into());
        }
        if !assignment.output.is_empty() {
            msg!("output: {:?}", assignment.output);
            return Err(SolLearnError::Unauthorized.into());
        }
        if inference.status != 1 {
            msg!("status: {}", inference.status);
            return Err(SolLearnError::Unauthorized.into());
        }

        let mut concatenated: Vec<u8> = infer_id.to_le_bytes().to_vec();
        concatenated.extend(data.clone());
        let digest = hash(&mut concatenated);
        assignment.digest = digest.to_bytes();
        assignment.commitment = digest.to_bytes();
        assignment.output = data.clone();

        inference.status = 2;
        inference.assignments.push(assignment.id);
        inference.digests.values.push(digest.to_bytes());

        


        emit!(SolutionSubmission {
            assignment_id,
            sender: ctx.accounts.signer.key(),
        });
        emit!(InferenceStatusUpdate {
            inference_id: infer_id,
            status: 2,
        });

        Ok(())
    }

    pub fn commit(
        ctx: Context<UpdateAssignmentVld>,
        assignment_id: u64,
        inference_id: u64,
        commitment: [u8; 32],
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;
        let assignment = &mut ctx.accounts.assignment;
        let inference = &mut ctx.accounts.infs;
        let voting_info = &mut ctx.accounts.voting_info;

        let infer_id = assignment.inference_id;

        if assignment_id != assignment.id {
            return Err(SolLearnError::Unauthorized.into());
        }
        if inference.id != infer_id {
            return Err(SolLearnError::Unauthorized.into());
        }
        if ctx.accounts.signer.key() != assignment.worker {
            return Err(SolLearnError::Unauthorized.into());
        }
        if assignment.role != 1 {
            return Err(SolLearnError::Unauthorized.into());
        }
        if assignment.commitment != [0; 32] {
            return Err(SolLearnError::Unauthorized.into());
        }
        if inference.status != 2 {
            return Err(SolLearnError::Unauthorized.into());
        }

        let slot_number = Clock::get()?.slot;
        if slot_number > inference.commit_timeout {
            return Err(SolLearnError::Unauthorized.into());
        }

        assignment.commitment = commitment;
        let l = inference.assignments.len();
        inference.assignments.insert(l, assignment.id);
        voting_info.total_commit += 1;

        if voting_info.total_commit as usize == inference.assignments.len() - 1 {
            inference.status = 3;
            emit!(InferenceStatusUpdate {
                inference_id: infer_id,
                status: 3,
            });
        }
        emit!(CommitmentSubmission {
            assignment_id,
            sender: ctx.accounts.signer.key(),
            commitment,
        });

        Ok(())
    }

    pub fn reveal(
        ctx: Context<UpdateAssignmentVld>,
        assignment_id: u64,
        inference_id: u64,
        nonce: u64,
        data: Vec<u8>,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;
        let assignment = &mut ctx.accounts.assignment;
        let inference = &mut ctx.accounts.infs;
        let voting_info = &mut ctx.accounts.voting_info;

        let infer_id = assignment.inference_id;
        if assignment_id != assignment.id {
            return Err(SolLearnError::Unauthorized.into());
        }
        if inference.id != infer_id {
            return Err(SolLearnError::Unauthorized.into());
        }
        if ctx.accounts.signer.key() != assignment.worker {
            return Err(SolLearnError::Unauthorized.into());
        }
        if assignment.role != 1 {
            return Err(SolLearnError::Unauthorized.into());
        }
        if assignment.commitment == [0; 32] {
            return Err(SolLearnError::Unauthorized.into());
        }
        let slot_number = Clock::get()?.slot;
        if slot_number > inference.reveal_timeout {
            return Err(SolLearnError::Unauthorized.into());
        }
        if inference.status == 2 {
            inference.status = 3;
        } else if inference.status != 3 {
            return Err(SolLearnError::Unauthorized.into());
        }

        let mut concatenated: Vec<u8> = nonce.to_le_bytes().to_vec();
        let sender_key = ctx.accounts.signer.key().clone();
        concatenated.extend(ctx.accounts.signer.key().to_bytes().to_vec());
        concatenated.extend(data.clone());
        let reveal_hash = hash(&mut concatenated);
        if assignment.commitment != reveal_hash.to_bytes() {
            return Err(SolLearnError::InvalidReveal.into());
        }

        let digest = hash(&mut [infer_id.to_le_bytes().to_vec(), data.clone()].concat());
        assignment.reveal_nonce = nonce;
        assignment.output = data.clone();
        assignment.digest = digest.to_bytes();
        voting_info.total_reveal += 1;

        if inference.digests.values.len() == 0 {
            let zero: [u8; 32] = [0; 32];
            inference.digests.values = vec![zero; inference.assignments.len()];
        }

        let index = inference
            .assignments
            .iter()
            .position(|&r| r == assignment_id)
            .unwrap();
        if inference.digests.values[index] == [0; 32] {
            inference.digests.values[index] = digest.to_bytes();
        } else {
            return Err(SolLearnError::InvalidReveal.into());
        }

        if voting_info.total_reveal as usize == inference.assignments.len() - 1 {
            resolve_inference(ctx, assignment_id, infer_id)?;
        }

        emit!(RevealSubmission {
            assignment_id,
            sender: sender_key,
            nonce,
            data,
        });

        Ok(())
    }

    pub fn resolve_inference(ctx: Context<UpdateAssignmentVld>, assignment_id: u64, inference_id: u64) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        let inference = &mut ctx.accounts.infs;
        let assignment = &mut ctx.accounts.assignment;
        let dao_receivers = &mut ctx.accounts.dao_receiver_infos;
        let voting_info = &mut ctx.accounts.voting_info;

        if ctx.accounts.token_recipient.key() != inference.creator {
            return Err(SolLearnError::WrongRecipient.into());
        }

        if assignment_id != assignment.id {
            return Err(SolLearnError::Unauthorized.into());
        }
        // only_updated_epoch(acc)?;

        let infer_id = inference.id;
        if inference.id != infer_id || inference.id != inference_id {
            return Err(SolLearnError::Unauthorized.into());
        }

        if inference.status == 1 {
            if Clock::get()?.slot > inference.submit_timeout
                && inference.processed_miner != Pubkey::default()
            {
                inference.status = 5;

                let value = inference.value + inference.fee_l2 + inference.fee_treasury;
                let cpi_accounts = Transfer {
                    from: ctx.accounts.vault_staking_wallet.to_account_info(),
                    to: ctx.accounts.token_recipient.to_account_info(),
                    authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                token::transfer(cpi_ctx, value)?;

                // let from = ctx.accounts.vault_wallet_owner_pda.to_account_info();
                // let to = ctx.accounts.recipient.to_account_info();
                // if **from.try_borrow_lamports()? < value {
                //     return Err(SolLearnError::InsufficientFunds.into());
                // }
                // **from.try_borrow_mut_lamports()? -= value;
                // **to.try_borrow_mut_lamports()? += value;

                // _slash_miner(inference.processedMiner, true);
                let tasks = &mut ctx.accounts.tasks;
                let mut data = vec![];
                data.push(1);
                data.extend_from_slice(&inference.processed_miner.to_bytes());

                tasks.push_task(Task::new(FnType::SlashMiner, data));

            }
        } else if inference.status == 2 {
            msg!("commit_timeout: {} vs {}", inference.commit_timeout, Clock::get()?.slot);
            if Clock::get()?.slot > inference.commit_timeout {
                msg!("total_commit: {}", voting_info.total_commit);
                if voting_info.total_commit + 1 >= inference.assignments.len() as u8 {
                    inference.status = 3;
                } else {
                    inference.status = 4;
                    let value = inference.value + inference.fee_l2 + inference.fee_treasury;
                    let cpi_accounts = Transfer {
                        from: ctx.accounts.vault_staking_wallet.to_account_info(),
                        to: ctx.accounts.token_recipient.to_account_info(),
                        authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
                    };
                    let cpi_program = ctx.accounts.token_program.to_account_info();
                    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                    token::transfer(cpi_ctx, value)?;

                    // let from = ctx.accounts.vault_wallet_owner_pda.to_account_info();
                    // let to = ctx.accounts.recipient.to_account_info();
                    // if **from.try_borrow_lamports()? < value {
                    //     return Err(SolLearnError::InsufficientFunds.into());
                    // }
                    // **from.try_borrow_mut_lamports()? -= value;
                    // **to.try_borrow_mut_lamports()? += value;

                    for i in 0..inference.assignments.len() {
                        // _slash_miner(assignment.worker, false);
                        // create new task
                        let tasks = &mut ctx.accounts.tasks;
                        let mut data = vec![];
                        data.push(0);
                        data.extend_from_slice(&inference.assignments[i].to_le_bytes());
                        data.push(0);
                        data.push(1);
                        data.push(0);
                        tasks.push_task(Task::new(FnType::SlashMiner, data));
                    }
                }
                msg!("final inference status {}", inference.status);
            }
        }
        if inference.status == 3 {
            if Clock::get()?.slot > inference.reveal_timeout
                || voting_info.total_reveal == voting_info.total_commit
            {
                let tasks = &mut ctx.accounts.tasks;
                if !filter_commitment(acc, inference, assignment, dao_receivers, tasks)? {
                    msg!("afdasfasfsa 1");
                    //  handle_not_enough_vote(ctx.accounts.infs.id);
                    let value = inference.value + inference.fee_l2 + inference.fee_treasury;
                    let cpi_accounts = Transfer {
                        from: ctx.accounts.vault_staking_wallet.to_account_info(),
                        to: ctx.accounts.token_recipient.to_account_info(),
                        authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
                    };
                    let cpi_program = ctx.accounts.token_program.to_account_info();
                    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                    msg!("afdasfasfsa 2");
                    token::transfer(cpi_ctx, value)?;
                    msg!("afdasfasfsa 3");

                    // let from = ctx.accounts.vault_wallet_owner_pda.to_account_info();
                    // let to = ctx.accounts.recipient.to_account_info();
                    // if **from.try_borrow_lamports()? < value {
                    //     return Err(SolLearnError::InsufficientFunds.into());
                    // }
                    // **from.try_borrow_mut_lamports()? -= value;
                    // **to.try_borrow_mut_lamports()? += value;

                    for i in 0..inference.assignments.len() {
                        let dig = inference.digests.values[i];
                        if dig == [0; 32] {
                            // _slash_miner(ctx, ctx.accounts.assignments[assignment_id].worker, false)?;

                            let mut data = vec![];
                            data.push(0);
                            data.extend_from_slice(&inference.assignments[i].to_le_bytes());
                            data.push(0);
                            data.push(0);
                            data.push(0);
                            tasks.push_task(Task::new(FnType::SlashMiner, data));
                        }
                    }
                    inference.status = 4;
                }
            }
        }

        emit!(InferenceStatusUpdate {
            inference_id: infer_id,
            status: inference.status,
        });

        Ok(())
    }

    pub fn pay_miner(ctx: Context<PayMinerVld>, assignment_id: u64) -> Result<()> {
        let tasks = &mut ctx.accounts.tasks;
        let assignment = &mut ctx.accounts.assignment;

        let task;
        match tasks.pop_task() {
            Some(t) => task = t,
            None => return Err(SolLearnError::NoValidTask.into()),
        }
        if task.fn_type() != FnType::PayMiner {
            return Err(SolLearnError::NoValidTask.into());
        }
        let data = task.data();
        let use_assignment = data[0] == 1;
        let value = if use_assignment {
            let _assignment_id = u64::from_le_bytes(data[1..9].try_into().unwrap());

            let pubkey = assignment.worker;
            let mut value_bytes = [0u8; 8];
            value_bytes.copy_from_slice(&data[9..17]);
            let v = u64::from_le_bytes(value_bytes);
            if ctx.accounts.token_recipient.key() != pubkey {
                return Err(SolLearnError::WrongRecipient.into());
            }
            let set_vote = data[16];
            if set_vote > 0 {
                assignment.vote = set_vote;
            }
            v
        } else {
            let mut pubkey_bytes = [0u8; 32];
            pubkey_bytes.copy_from_slice(&data[1..33]);
            let pubkey = Pubkey::new_from_array(pubkey_bytes);
            let mut value_bytes = [0u8; 8];
            value_bytes.copy_from_slice(&data[33..41]);
            let v = u64::from_le_bytes(value_bytes);
            if ctx.accounts.token_recipient.key() != pubkey {
                return Err(SolLearnError::WrongRecipient.into());
            }
            v
        };
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_staking_wallet.to_account_info(),
            to: ctx.accounts.token_recipient.to_account_info(),
            authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, value)?;

        // let from = ctx.accounts.vault_wallet_owner_pda.to_account_info();
        // let to = ctx.accounts.recipient.to_account_info();
        // if **from.try_borrow_lamports()? < value {
        //     return Err(SolLearnError::InsufficientFunds.into());
        // }
        // **from.try_borrow_mut_lamports()? -= value;
        // **to.try_borrow_mut_lamports()? += value;

        Ok(())
    }

    pub fn slash_miner_by_admin(
        ctx: Context<SlashMinerByAdminVld>,
        _miner: Pubkey,
        is_fined: bool,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        if _miner == Pubkey::default() {
            return Err(SolLearnError::Unauthorized.into());
        }

        let miner_addresses = &mut ctx.accounts.miners_of_model;
        let miner = &mut ctx.accounts.miner_account;
        if miner.miner != _miner {
            return Err(SolLearnError::Unauthorized.into());
        }

        _slash_miner(miner, is_fined, acc, miner_addresses)?;

        Ok(())
    }

    pub fn slash_miner(ctx: Context<SlashMinerVld>, assignment_id: u64) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        let miner_addresses = &mut ctx.accounts.miners_of_model;
        let miner = &mut ctx.accounts.miner_account;
        let assignment = &mut ctx.accounts.assignment;

        let tasks = &mut ctx.accounts.tasks;
        let task;
        match tasks.pop_task() {
            Some(t) => task = t,
            None => return Err(SolLearnError::NoValidTask.into()),
        }
        if task.fn_type() != FnType::SlashMiner {
            return Err(SolLearnError::NoValidTask.into());
        }
        let data = task.data();
        let slashing_processed_miner = data[0] == 1;
        let token_fine = if slashing_processed_miner {
            let mut pubkey_bytes = [0u8; 32];
            pubkey_bytes.copy_from_slice(&data[1..33]);
            let pubkey = Pubkey::new_from_array(pubkey_bytes);
            let is_fined = data[33] == 1;
            if pubkey != miner.miner {
                return Err(SolLearnError::Unauthorized.into());
            }

            _slash_miner(miner, is_fined, acc, miner_addresses)?
        } else {
            let _assignment_id: u64 = u64::from_le_bytes(data[1..9].try_into().unwrap());
            let is_fined = data[9] == 1;
            let check_empty_commit = data[10] == 1;
            if assignment_id != assignment.id || _assignment_id != assignment_id {
                return Err(SolLearnError::Unauthorized.into());
            }
            if check_empty_commit {
                if assignment.commitment != [0; 32] {
                    return Ok(()); // not slashed
                }
            }

            let set_vote = data[11];
            if set_vote > 0 {
                assignment.vote = set_vote;
            }
            let pubkey = assignment.worker;
            if pubkey != miner.miner {
                return Err(SolLearnError::Unauthorized.into());
            }
            _slash_miner(miner, is_fined, acc, miner_addresses)?
        };
        if token_fine > 0 {
            if ctx.accounts.sol_learn_account.treasury == ctx.accounts.token_recipient.key() {
                return Err(SolLearnError::Unauthorized.into());
            }
            let cpi_accounts = Transfer {
                from: ctx.accounts.vault_staking_wallet.to_account_info(),
                to: ctx.accounts.token_recipient.to_account_info(),
                authority: ctx.accounts.vault_wallet_owner_pda.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::transfer(cpi_ctx, token_fine)?;
        }

        Ok(())
    }

    pub fn calculate_user_dao_token_received(
        ctx: Context<UpdateEpochVld>,
        score: u8,
    ) -> Result<u64> {
        let acc = &ctx.accounts.sol_learn_account;
        let mut user_dao_token_receive = 0;

        if score >= 1 && score <= 10 {
            user_dao_token_receive = (((acc.dao_token_percentage.user_percentage as u64)
                * ((score as u64) * (acc.dao_token_reward as u64)))
                / 10)
                / PERCENTAGE_DENOMINATOR;
        }

        Ok(user_dao_token_receive.into())
    }

    pub fn set_fine_percentage(ctx: Context<UpdateParamsVld>, fine_percentage: u16) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        acc.fine_percentage = fine_percentage;
        emit!(FinePercentageUpdated {
            new_fine_percentage: fine_percentage,
        });

        Ok(())
    }

    pub fn set_penalty_duration(
        ctx: Context<UpdateParamsVld>,
        penalty_duration: u64,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        acc.penalty_duration = penalty_duration;
        emit!(PenaltyDurationUpdated {
            new_penalty_duration: penalty_duration,
        });

        Ok(())
    }

    pub fn set_min_fee_to_use(ctx: Context<UpdateParamsVld>, min_fee_to_use: u64) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        acc.min_fee_to_use = min_fee_to_use;
        emit!(MinFeeToUseUpdated {
            new_min_fee_to_use: min_fee_to_use,
        });

        Ok(())
    }

    pub fn set_l2_owner(ctx: Context<UpdateParamsVld>, l2_owner_address: Pubkey) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        acc.l2_owner = l2_owner_address;
        emit!(L2OwnerUpdated {
            new_l2_owner: l2_owner_address,
        });

        Ok(())
    }

    // pub fn set_dao_token(ctx: Context<UpdateParamsVld>, dao_token_address: Pubkey) -> Result<()> {
    //     let acc = &mut ctx.accounts.sol_learn_account;
    //     only_updated_epoch(acc)?;

    //     acc.dao_token = dao_token_address;
    //     emit!(DaoTokenUpdated {
    //         new_dao_token: dao_token_address,
    //     });

    //     Ok(())
    // }

    pub fn set_treasury_address(
        ctx: Context<UpdateParamsVld>,
        treasury_address: Pubkey,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        acc.treasury = treasury_address;
        emit!(TreasuryAddressUpdated {
            new_treasury: treasury_address,
        });

        Ok(())
    }

    pub fn set_fee_ratio_miner_validator(
        ctx: Context<UpdateParamsVld>,
        new_ratio: u16,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        acc.fee_ratio_miner_validator = new_ratio;
        emit!(FeeRatioMinerValidatorUpdated {
            new_fee_ratio_miner_validator: new_ratio as u64,
        });

        Ok(())
    }

    pub fn set_dao_token_reward(
        ctx: Context<UpdateParamsVld>,
        new_dao_token_reward: u64,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.sol_learn_account;
        // only_updated_epoch(acc)?;

        acc.dao_token_reward = new_dao_token_reward;
        emit!(DaoTokenRewardUpdated {
            new_dao_token_reward,
        });

        Ok(())
    }
}

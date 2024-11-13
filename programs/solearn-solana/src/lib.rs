pub mod errors;
pub mod state;
pub mod state_inf;
mod utils;

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, TransferChecked, transfer_checked};
use anchor_lang::solana_program::keccak::hash;
use anchor_lang::system_program;
use state::*;
use state_inf::*;
use utils::*;
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
        let model_index = random_number(&&Clock::get()?, 0, (ctx.accounts.models.data.len() / 32) as u64);
        let model: Pubkey = ctx.accounts.models.data[model_index as usize * 32..(model_index + 1) as usize * 32].try_into().expect("Invalid length");
        miner_info.model = model;
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

        if ctx.accounts.miner_account.is_active {
            return Err(SolLearnError::Activated.into())
        }

        // insert model address
        let miners_of_model = &mut ctx.accounts.miners_of_model;
        miners_of_model.data.extend_from_slice(ctx.accounts.miner.key().as_ref());

        // update miner join time
        ctx.accounts.miner_account.last_time = ctx.accounts.sysvar_clock.unix_timestamp as u64;
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

        // Assuming _updateEpoch() is a function that updates the epoch based on the current clock
        // _update_epoch(&ctx.accounts.sysvar_clock)?;

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

        if ctx.accounts.miner_account.model_index != 0 {
            return Err(SolLearnError::MinerNotRegistered.into())
        }

        if ctx.accounts.miner_account.unstaking_time != 0 {
            return Err(SolLearnError::Unstaked.into())
        }

        // update account unstaking time
        ctx.accounts.miner_account.unstaking_time = (ctx.accounts.sysvar_clock.unix_timestamp as u64) + ctx.accounts.sol_learn_account.unstake_delay_time;
        if ctx.accounts.miner_account.is_active {
            ctx.accounts.miner_account.is_active = false;

            // todo: update epoch reward here
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

    // claim 
    pub fn miner_claim(ctx: Context<MinerClaim>) -> Result<()> {
        
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
            &b"vault"[..], solean_key.as_ref()
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

    // slash miner
    // claim reward
    // epoch update
    // set fine percentage
    // setPenaltyDuration
    // setMinFeeToUse
    // setNewRewardInEpoch

    pub fn set_miner_minimum_stake(ctx: Context<UpdateParamsVld>, data: u64) -> Result<()> {
        let acc = &mut ctx.accounts.wh_account;
        acc.miner_minimum_stake = data.into();
        Ok(())
    }

    pub fn infer(
        ctx: Context<InferVld>,
        input: Vec<u8>,
        creator: Pubkey,
        _value: u64,
    ) -> Result<u64> {
        let acc = &mut ctx.accounts.wh_account;
        let models_map = get_models_as_map(&acc.models);
        let model = models_map.get(&creator).unwrap();
        if model.tier == 0 {
            return Err(SolLearnError::Unauthorized.into());
        }

        let scoring_fee = validate_enough_fee_to_use(model.minimum_fee, _value)?; // TODO

        if ctx.accounts.recipient.key() != *ctx.program_id {
            return Err(SolLearnError::WrongRecipient.into());
        }
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.recipient.to_account_info(),
                },
            ),
            _value,
        )?;
        let value = _value - scoring_fee;

        acc.inference_number += 1;
        let inference_id = acc.inference_number;
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
        // inference.referrer = referrer_of[creator];
        inference.model_address = ctx.accounts.signer.key();
        inference.bump = ctx.bumps.infs;

        // emit NewInference(inference_id, msg.sender, creator, value, 0);

        // assign_miners(inference_id, acc, inference)?;
        let slot_number = Clock::get()?.slot;
        let expired_at = slot_number + acc.submit_duration;
        let commit_timeout = expired_at + acc.commit_duration;
        inference.submit_timeout = expired_at;
        inference.commit_timeout = commit_timeout;
        inference.reveal_timeout = commit_timeout + acc.reveal_duration;
        inference.status = 1;

        let model = inference.model_address;
        let miner_addresses = &mut ctx.accounts.miner_addresses;
        let n = acc.miner_requirement;
        let mut selected_miners = Vec::with_capacity(n as usize);

        for i in 0..n {
            // randomizer.random_uint256()
            let rand_uint = random_number(&&Clock::get()?, i.into(), miner_addresses.values.len() as u64);
            
            let miner_ind = (rand_uint as usize) % miner_addresses.values.len();
            let miner = miner_addresses.values[miner_ind];
            let assignment = &mut ctx.accounts.assignment;
            // miner_addresses.erase(miner);
            miner_addresses.values.remove(miner_ind);
            let assignment_id = acc.assignment_number;
            acc.assignment_number += 1;
            assignment.inference_id = inference_id;
            assignment.worker = miner;
            assignment.role = 1;

            selected_miners.push(miner);
            // assignments_by_miner[miner].insert(assignment_id);
            // assignments_by_inference[inference_id].insert(assignment_id);
            // emit NewAssignment(assignment_id, inference_id, miner, expired_at);
        }

        for miner in selected_miners {
            let current_len = miner_addresses.values.len();
            miner_addresses.values.insert(current_len, miner);
        }

        Ok(0)
    }

    pub fn top_up_infer(ctx: Context<UpdateInferVld>, value: u64) -> Result<()> {
        if value == 0 {
            return Err(SolLearnError::ZeroValue.into());
        }

        if ctx.accounts.recipient.key() != *ctx.program_id {
            return Err(SolLearnError::WrongRecipient.into());
        }
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.recipient.to_account_info(),
                },
            ),
            value,
        )?;

        let inference = &mut ctx.accounts.infs;
        if inference.status != 1 {
            return Err(SolLearnError::InferMustBeSolvingState.into());
        }

        inference.value += value;

        Ok(())
    }

    pub fn seize_miner_role(ctx: Context<UpdateAssignmentVld>, assignment_id: u64) -> Result<()> {
        let acc = &mut ctx.accounts.wh_account;
        let inference = &mut ctx.accounts.infs;
        let assignment = &mut ctx.accounts.assignment;
        let miner_epoch_state = &mut ctx.accounts.miner_reward;

        update_epoch(acc, miner_epoch_state)?;

        if assignment.worker != ctx.accounts.signer.key() {
            return Err(SolLearnError::Unauthorized.into());
        }

        let _infer_id = assignment.inference_id;
        if inference.processed_miner != Pubkey::default() {
            return Err(SolLearnError::InferenceSeized.into());
        }

        assignment.role = 2;
        inference.processed_miner = ctx.accounts.signer.key();

        Ok(())
    }

    pub fn submit_solution(
        ctx: Context<UpdateAssignmentVld>,
        assignment_id: u64,
        data: Vec<u8>,
    ) -> Result<()> {
        let acc = &mut ctx.accounts.wh_account;
        let miner_epoch_state = &mut ctx.accounts.miner_reward;
        update_epoch(acc, miner_epoch_state)?;
        let assignment = &mut ctx.accounts.assignment;
        let inference = &mut ctx.accounts.infs;

        let infer_id = assignment.inference_id;

        if inference.id != infer_id {
            return Err(SolLearnError::Unauthorized.into());
        }
        if ctx.accounts.signer.key() != assignment.worker {
            return Err(SolLearnError::Unauthorized.into());
        }
        if assignment.role != 1 {
            return Err(SolLearnError::Unauthorized.into());
        }
        if !assignment.output.is_empty() {
            return Err(SolLearnError::Unauthorized.into());
        }
        if inference.status != 1 {
            return Err(SolLearnError::Unauthorized.into());
        }

        let mut concatenated: Vec<u8> = infer_id.to_le_bytes().to_vec();
        concatenated.extend(data);
        let digest = hash(&mut concatenated);
        assignment.digest = digest.to_bytes();
        assignment.commitment = digest.to_bytes();
        inference.status = 2;
        inference.assignments.push(assignment.id);

        Ok(())
    }

    pub fn commit(ctx: Context<UpdateAssignmentVld>, commitment: [u8; 32]) -> Result<()> {
        let acc = &mut ctx.accounts.wh_account;
        let miner_epoch_state = &mut ctx.accounts.miner_reward;
        update_epoch(acc, miner_epoch_state)?;
        let assignment = &mut ctx.accounts.assignment;
        let inference = &mut ctx.accounts.infs;
        let voting_info = &mut ctx.accounts.voting_info;

        let infer_id = assignment.inference_id;

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
        }

        Ok(())
    }

    pub fn reveal(ctx: Context<UpdateAssignmentVld>, nonce: u64, data: Vec<u8>) -> Result<()> {
        let acc = &mut ctx.accounts.wh_account;
        let miner_epoch_state = &mut ctx.accounts.miner_reward;
        update_epoch(acc, miner_epoch_state)?;
        let assignment = &mut ctx.accounts.assignment;
        let inference = &mut ctx.accounts.infs;
        let voting_info = &mut ctx.accounts.voting_info;

        let infer_id = assignment.inference_id;

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

        let list_digests = &mut ctx.accounts.digests;
        if !list_digests.values.contains(&digest.to_bytes()) {
            list_digests.values.push(digest.to_bytes());
        }

        if voting_info.total_reveal as usize == inference.assignments.len() - 1 {
            resolve_inference(ctx)?;
        }

        Ok(())
    }

    pub fn resolve_inference(ctx: Context<UpdateAssignmentVld>) -> Result<()> {
        let acc = &mut ctx.accounts.wh_account;
        let miner_epoch_state = &mut ctx.accounts.miner_reward;
        update_epoch(acc, miner_epoch_state)?;
        let inference = &mut ctx.accounts.infs;
        let voting_info = &mut ctx.accounts.voting_info;
        // let digests = &mut ctx.accounts.digests;

        let infer_id = inference.id;

        if inference.status == 1 && Clock::get()?.slot > inference.submit_timeout {
            if inference.processed_miner != Pubkey::default() {
                inference.status = 5;
                // system_program::transfer(
                //     CpiContext::new(
                //         ctx.accounts.system_program.to_account_info(),
                //         system_program::Transfer {
                //             from: ctx.accounts.recipient.to_account_info(),
                //             to: ctx.accounts.infs.creator.to_account_info(),
                //         },
                //     ),
                //     inference.value + inference.fee_l2 + inference.fee_treasury,
                // )?;
                // _slash_miner(inference.processedMiner, true);
            }
        }

        if inference.status == 2 && Clock::get()?.slot > inference.commit_timeout {
            if voting_info.total_commit + 1 >= inference.assignments.len() as u8 {
                inference.status = 3;
            } else {
                inference.status = 4;
                // system_program::transfer(
                //     CpiContext::new(
                //         ctx.accounts.system_program.to_account_info(),
                //         system_program::Transfer {
                //             from: ctx.accounts.recipient.to_account_info(),
                //             to: ctx.accounts.infs.creator.to_account_info(),
                //         },
                //     ),
                //     inference.value + inference.fee_l2 + inference.fee_treasury,
                // )?;
                for assignment_id in &inference.assignments {
                    let assignment = &ctx.accounts.assignment;
                    if assignment.commitment == [0; 32] {
                        // _slash_miner(assignment.worker, false);
                    }
                }
            }
        }

        if inference.status == 3 {
            if Clock::get()?.slot > inference.reveal_timeout
                || voting_info.total_reveal == voting_info.total_commit
            {
                // if !filter_commitment(ctx.accounts.infs.id, digests) {
                //     handle_not_enough_vote(ctx.accounts.infs.id);
                //     inference.status = 4;
                // }
            }
        }

        Ok(())
    }

    pub fn slash_miner(ctx: Context<UpdateEpochVld>, _miner: Pubkey, is_fined: bool) -> Result<()> {
        let acc = &mut ctx.accounts.wh_account;
        let miner_epoch_state = &mut ctx.accounts.miner_reward;
        update_epoch(acc, miner_epoch_state)?;

        if _miner == Pubkey::default() {
            return Err(SolLearnError::Unauthorized.into());
        }

        let miner_addresses = &mut ctx.accounts.miner_addresses;
        let miner = &mut ctx.accounts.miner;
        if miner.address != _miner {
            return Err(SolLearnError::Unauthorized.into());
        }


        _slash_miner(miner, is_fined, acc, miner_addresses)?;

        Ok(())
    }

}

fn random_number(clk: &Clock, nonce: u64, range: u64) -> u64 {
    if range == 0 {
        return 0;
    }
    // tbd: get recent blockhash
    // let data = recent_slothashes.data.borrow();
    // let most_recent = array_ref![data, 12, 8];
    let mut cloned_data: Vec<u8> = vec![];
    let nonce_bytes = nonce.to_le_bytes();
    cloned_data.extend_from_slice(&nonce_bytes);
    let time_bytes = (clk.unix_timestamp as u64) as u64;
    cloned_data.extend_from_slice(&time_bytes.to_le_bytes());
    hash(&cloned_data);
    let rightmost: &[u8] = &cloned_data[28..];
    
    let seed = u64::try_from_slice(rightmost).unwrap();
    
    seed % range

    
    // let leader_schedule_epoch = clk.leader_schedule_epoch;
    // let most_recent = array_ref![recent_blockhash_data, 0, 16];
    // u128::from_le_bytes(*most_recent)
    // leader_schedule_epoch % range
}

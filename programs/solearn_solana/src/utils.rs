use std::collections::HashMap;

use crate::errors::*;
use crate::state_inf::*;
use crate::MinerInfo;
use crate::MinersOfModel;
use crate::SolLearnInfo;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hash;

pub const PERCENTAGE_DENOMINATOR: u64 = 100_00;
pub const BLOCK_PER_YEAR: u64 = 365 * 24 * 60 * 60 / 2; // 2s per block

pub fn validate_enough_fee_to_use(minimum_fee: u64, value: u64) -> Result<u64> {
    if value < minimum_fee {
        return Err(SolLearnError::FeeTooLow.into());
    }

    Ok(minimum_fee)
}

// pub fn update_epoch(es: &mut SolLearnInfo, ms: &mut MinerEpochState) -> Result<()> {
//     let slot_number = Clock::get()?.slot;
//     let epoch_passed = (slot_number - es.last_block) / es.blocks_per_epoch;
//     if epoch_passed > 0 {
//         es.last_block += es.blocks_per_epoch * epoch_passed;
//         let reward_in_current_epoch = (es.reward_per_epoch * es.blocks_per_epoch) / BLOCK_PER_YEAR;

//         for _ in 0..epoch_passed {
//             ms.total_miner = es.miner_addresses.values.len() as u64;
//             ms.epoch_reward = reward_in_current_epoch;
//             es.current_epoch += 1;
//         }
//     } else {
//         es.last_block = slot_number;
//     }
//     Ok(())
// }

pub fn only_updated_epoch(es: &mut SolLearnInfo) -> Result<()> {
    let slot_number = Clock::get()?.slot;
    let epoch_passed = (slot_number - es.last_block) / es.blocks_per_epoch;
    if epoch_passed > 0 {
        return Err(SolLearnError::NeedToWait.into());
    }
    Ok(())
}

pub fn only_empty_tasks(t: &mut Tasks) -> Result<()> {
    if t.values.len() > 0 {
        return Err(SolLearnError::MustWaitTasks.into());
    }
    Ok(())
}

pub fn _slash_miner(
    miner: &mut MinerInfo,
    is_fined: bool,
    acc: &mut SolLearnInfo,
    miner_addresses: &mut MinersOfModel,
) -> Result<u64> {
    // if !acc.miner_addresses.values.contains(&miner.miner) {
    //     return Err(SolLearnError::Unauthorized.into());
    // }

    // _claim_reward(miner, false);
    let mut remove_ind = 0;
    for i in 0..(miner_addresses.data.len()/32) {
        let pubkey_bytes: [u8; 32] = miner_addresses.data[i*32..(i+1)*32].try_into().unwrap();
        
        if pubkey_bytes == miner.miner.to_bytes() {
            remove_ind = i;
            break;
        }
    }
    miner_addresses.data.drain(remove_ind*32..(remove_ind+1)*32);
    
    miner.active_time = Clock::get()?.slot + acc.penalty_duration;
    emit!(MinerDeactivated {
        miner: miner.miner,
        model_address: miner.miner,
        active_time: miner.active_time,
    });



    if is_fined {
        let fine = (acc.miner_minimum_stake * acc.fine_percentage as u64) / PERCENTAGE_DENOMINATOR;
        if miner.stake_amount < fine {
            miner.stake_amount = 0;
        } else {
            miner.stake_amount -= fine;
        }
        emit!(MinerPenalized {
            miner: miner.miner,
            model_address: miner.miner,
            treasury: acc.treasury,
            fine,
        });

        Ok(fine)

        // boost[_miner].reserved1 = 0;
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
    } else {
        Ok(0)
    }
}

pub fn calculate_transferred_dao_token(
    acc: &mut SolLearnInfo,
    inference: &mut Inference,
    is_referred: bool,
) -> Result<()> {
    let l2_owner_amt = (acc.dao_token_reward
        * u64::from(acc.dao_token_percentage.l2_owner_percentage))
        / PERCENTAGE_DENOMINATOR;
    // dao_receivers.values = Vec::new();
    // dao_receivers.values.insert(
    //     0,
    //     DAOTokenReceiverInfo {
    //         receiver: acc.l2_owner,
    //         amount: l2_owner_amt,
    //         role: 5, // DAOTokenReceiverRole::L2Owner
    //     },
    // );

    if is_referred {
        let referee_amt = (acc.dao_token_reward
            * u64::from(acc.dao_token_percentage.referee_percentage))
            / PERCENTAGE_DENOMINATOR;
        let referer_amt = (acc.dao_token_reward
            * u64::from(acc.dao_token_percentage.referrer_percentage))
            / PERCENTAGE_DENOMINATOR;

        // dao_receivers.values.insert(
        //     0,
        //     DAOTokenReceiverInfo {
        //         receiver: inference.creator,
        //         amount: referee_amt,
        //         role: 4, // DAOTokenReceiverRole::Referee
        //     },
        // );

        // dao_receivers.values.insert(
        //     0,
        //     DAOTokenReceiverInfo {
        //         receiver: inference.referrer,
        //         amount: referer_amt,
        //         role: 3, // DAOTokenReceiverRole::Referrer
        //     },
        // );
    }

    Ok(())
}

pub fn filter_commitment(
    acc: &mut SolLearnInfo,
    inference: &mut Inference,
    assignment: &mut Assignment,
    tasks: &mut Tasks,
) -> Result<bool> {
    // let acc = &mut ctx.accounts.wh_account;
    // let inference = &mut ctx.accounts.infs;
    // let assignment = &ctx.accounts.assignment;
    // let dao_receivers = &mut ctx.accounts.dao_receiver_infos;
    let digests = &inference.digests;

    let (most_voted_digest, max_count) = find_most_voted_digest(digests.values.clone())?;
    if (max_count as u64) < get_threshold_value(inference.assignments.len() as u64) {
        return Ok(false);
    }

    let is_referred = inference.referrer != Pubkey::default();
    let not_reached_limit = true; // validate_dao_supply_increase(is_referred);

    let is_match_miner_result = assignment.digest == most_voted_digest;

    let mut fee_for_miner = 0;
    let mut share_fee_per_validator = 0;
    let remain_value = inference.value;
    let mut token_for_miner = 0;
    let mut share_token_per_validator = 0;
    let remain_token = ((acc.dao_token_percentage.miner_percentage as u64) * acc.dao_token_reward)
        / PERCENTAGE_DENOMINATOR;

    if not_reached_limit && remain_token > 0 {
        calculate_transferred_dao_token(acc, inference, is_referred)?;
    }

    if is_match_miner_result {
        fee_for_miner =
            (remain_value * acc.fee_ratio_miner_validator as u64) / PERCENTAGE_DENOMINATOR;
        share_fee_per_validator = (remain_value - fee_for_miner) / (max_count - 1);
        token_for_miner =
            (remain_token * acc.fee_ratio_miner_validator as u64) / PERCENTAGE_DENOMINATOR;
        share_token_per_validator = (remain_token - token_for_miner) / (max_count - 1);
    } else {
        share_fee_per_validator = remain_value / max_count;
        share_token_per_validator = remain_token / max_count;
    }

    for i in 0..inference.assignments.len() {
        // let assignment = &assignment[assignment_ids[i]];
        if inference.digests.values[i] != most_voted_digest {
            // assignment.vote = 1; // Vote::Disapproval
            // slash_miner(ctx, assignment.worker, true)?;
            let mut data = vec![];
            data.push(0);
            data.extend_from_slice(&inference.assignments[i].to_le_bytes());
            data.push(0);
            data.push(0);
            data.push(1);
            tasks.push_task(Task::new(FnType::SlashMiner, data));
        } else {
            // assignment.vote = 2; // Vote::Approval
            if assignment.role == 1 {
                // AssignmentRole::Validating
                if share_fee_per_validator > 0 {
                    let mut data = vec![];
                    data.push(1);
                    data.extend_from_slice(&assignment.id.to_le_bytes());
                    data.extend_from_slice(&share_fee_per_validator.to_le_bytes());
                    data.push(2);
                    tasks.push_task(Task::new(FnType::PayMiner, data));
                }
                if not_reached_limit && token_for_miner > 0 {
                    // dao_receivers.values.insert(
                    //     0,
                    //     DAOTokenReceiverInfo {
                    //         receiver: assignment.worker,
                    //         amount: share_token_per_validator,
                    //         role: 1, // DAOTokenReceiverRole::Validator
                    //     },
                    // );
                }
            } else {
                if fee_for_miner > 0 {
                    // TransferHelper.safeTransferNative(
                    //     assignment.worker,
                    //     fee_for_miner
                    // );
                    let mut data = vec![];
                    data.push(1);
                    data.extend_from_slice(&assignment.id.to_le_bytes());
                    data.extend_from_slice(&fee_for_miner.to_le_bytes());
                    data.push(2);
                    tasks.push_task(Task::new(FnType::PayMiner, data));
                }
                if not_reached_limit && token_for_miner > 0 {
                    // dao_receivers.values.insert(
                    //     0,
                    //     DAOTokenReceiverInfo {
                    //         receiver: assignment.worker,
                    //         amount: token_for_miner,
                    //         role: 0, // DAOTokenReceiverRole::Miner
                    //     },
                    // );
                }
            }
        }
    }

    // if not_reached_limit && dao_receivers.values.len() > 0 {
    //     let receivers_inf = dao_receivers;
    //     for i in 0..receivers_inf.values.len() {
            // IDAOToken(dao_token).mint(
            //     receivers_inf[i].receiver,
            //     receivers_inf[i].amount
            // );
        // }

        // emit DAOTokenMintedV2(
        //     _getChainID(),
        //     _inferenceId,
        //     inferences[_inferenceId].modelAddress,
        //     receiversInf
        // );
    // }

    if inference.fee_l2 > 0 {
        let mut data = vec![];
        data.push(0);
        data.extend(acc.l2_owner.to_bytes());
        data.extend_from_slice(&inference.fee_l2.to_le_bytes());
        tasks.push_task(Task::new(FnType::PayMiner, data));
    
    }
    if inference.fee_treasury > 0 {
        let mut data = vec![];
        data.push(0);
        data.extend(acc.treasury.to_bytes());
        data.extend_from_slice(&inference.fee_treasury.to_le_bytes());
        tasks.push_task(Task::new(FnType::PayMiner, data));
    }

    inference.status = 4;

    Ok(true)
}

pub fn find_most_voted_digest(list_digests: Vec<[u8; 32]>) -> Result<([u8; 32], u64)> {
    let mut max_count = 0;
    let mut most_voted_digest = list_digests[0];
    let mut counts: HashMap<[u8; 32], u64> = HashMap::new();

    for digest in list_digests.iter() {
        let count = *(counts.get(digest).unwrap_or(&0));
        counts.insert(*digest, count + 1);
        if count > max_count {
            max_count = count;
            most_voted_digest = *digest;
        }
    }

    Ok((most_voted_digest, max_count))
}

pub fn get_threshold_value(x: u64) -> u64 {
    (x * 2) / 3 + if x % 3 == 0 { 0 } else { 1 }
}

// pub fn get_models_as_map(models: &Vec<Model>) -> HashMap<Pubkey, Model> {
//     let mut map = HashMap::new();
//     for model in models {
//         let model = Model {
//             minimum_fee: model.minimum_fee,
//             tier: model.tier,
//             address: model.address,
//         };

//         map.insert(model.address, model);
//     }
//     map
// }

pub fn random_number(clk: &Clock, nonce: u64, range: u64) -> u64 {
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
    let res = hash(&cloned_data);
    let rightmost: &[u8] = &res.to_bytes()[24..];

    let seed = u64::try_from_slice(rightmost).unwrap();

    seed % range

    // let leader_schedule_epoch = clk.leader_schedule_epoch;
    // let most_recent = array_ref![recent_blockhash_data, 0, 16];
    // u128::from_le_bytes(*most_recent)
    // leader_schedule_epoch % range
}

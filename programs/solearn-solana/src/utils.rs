use std::collections::HashMap;

use anchor_lang::prelude::*;
use crate::errors::*;
use crate::state_inf::*;

pub const PERCENTAGE_DENOMINATOR: u64 = 100_00;
pub const BLOCK_PER_YEAR: u64 = 365 * 24 * 60 * 60 / 2; // 2s per block

pub fn validate_enough_fee_to_use(minimum_fee: u64, value: u64) -> Result<u64> {
    if value < minimum_fee {
        return Err(SolLearnError::FeeTooLow.into());
    }

    Ok(minimum_fee)
}

pub fn update_epoch(es: &mut WorkerHubStorage, ms: &mut MinerEpochState) -> Result<()> {
    let slot_number = Clock::get()?.slot;
    let epoch_passed = (slot_number - es.last_block) / es.blocks_per_epoch;
    if epoch_passed > 0 {
        es.last_block += es.blocks_per_epoch * epoch_passed;
        let reward_in_current_epoch = (es.reward_per_epoch * es.blocks_per_epoch) / BLOCK_PER_YEAR;

        for _ in 0..epoch_passed {
            // create multiple ?
            ms.total_miner = es.miner_addresses.values.len() as u64;
            ms.epoch_reward = reward_in_current_epoch;
            es.current_epoch += 1;
        }
    } else {
        es.last_block = slot_number;
    }
    Ok(())
}

pub fn _slash_miner(miner: &mut Worker, is_fined: bool, acc: &mut WorkerHubStorage, miner_addresses: &mut Pubkeys) -> Result<()> {
    if !acc.miner_addresses.values.contains(&miner.address) {
        return Err(SolLearnError::Unauthorized.into());
    }

    // _claim_reward(miner, false);
    let mut remove_ind = 0;
    for (i, m) in miner_addresses.values.iter().enumerate() {
        if *m == miner.address {
            remove_ind = i;
            break;
        }
    }
    miner_addresses.values.remove(remove_ind);
    miner.active_time = Clock::get()?.slot + acc.penalty_duration;

    if is_fined {
        let fine = (acc.miner_minimum_stake * acc.fine_percentage as u64) / PERCENTAGE_DENOMINATOR;
        if miner.stake < fine {
            miner.stake = 0;
        } else {
            miner.stake -= fine;
        }

        // TODO
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
    }

    Ok(())
}


pub fn get_models_as_map(models: &Vec<Model>) -> HashMap<Pubkey, Model> {
    let mut map = HashMap::new();
    for model in models {
        let model = Model {
            minimum_fee: model.minimum_fee,
            tier: model.tier,
            address: model.address,
        };

        map.insert(model.address, model);
    }
    map
}
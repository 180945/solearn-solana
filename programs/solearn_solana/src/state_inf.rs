use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{MinerInfo, MinersOfModel, Models, SolLearnInfo, VaultAccount};

#[derive(Accounts)]
pub struct UpdateParamsVld<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK:
    #[account(mut, constraint = sol_learn_account.admin == admin.key())]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReadStateVld<'info> {
    #[account(mut)]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
}

#[derive(Accounts)]
#[instruction(assignment_id: u64)]
pub struct ReadAssignmentVld<'info> {
    #[account(mut, seeds = [b"assignment", assignment_id.to_le_bytes().as_ref()], bump = assignment.bump)]
    pub assignment: Account<'info, Assignment>,
}

#[derive(Accounts)]
pub struct ReadTasksVld<'info> {
    #[account(mut)]
    pub tasks: Account<'info, Tasks>,
}

#[account]
pub struct Pubkeys {
    pub values: Vec<Pubkey>,
}

impl Pubkeys {
    pub fn len(&self) -> usize {
        8 + self.values.len() * 32
    }
}

// #[account]
// pub struct Models {
//     pub minimum_fee: u64,
//     pub tier: u32,
//     pub address: Pubkey,
// }
pub struct UnstakeRequest {}

pub enum InferenceStatus {
    Nil,
    Solving,
    Commit,
    Reveal,
    Processed,
    Killed,
    Transferred,
}

pub enum AssignmentRole {
    Nil,
    Validating,
    Mining,
}

#[account]
pub struct Inference {
    pub bump: u8,
    pub id: u64,
    pub assignments: Vec<u64>,
    pub digests: Hashes,
    pub input: Vec<u8>,
    pub value: u64,
    pub fee_l2: u64,
    pub fee_treasury: u64,
    pub model_address: Pubkey,
    pub submit_timeout: u64,
    pub commit_timeout: u64,
    pub reveal_timeout: u64,
    pub status: u8,
    pub creator: Pubkey,
    pub processed_miner: Pubkey,
    pub referrer: Pubkey,
}

#[account]
pub struct Referrer {
	pub bump: u8,
	pub pubkey: Pubkey,
}

#[derive(Accounts)]
#[instruction(inference_id: u64, creator: Pubkey)]
pub struct InferVld<'info> {
    #[account(
        init,
        payer = signer,
        space = 8*7 + 32*4 + 2 + 8 + 128, seeds = [b"inference", inference_id.to_le_bytes().as_ref()],
        bump
    )]
    pub infs: Account<'info, Inference>,
    #[account(mut)]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    // #[account(mut, seeds = [b"assignment", signer.key().as_ref()], bump = assignment.bump)]
    // pub assignment: Account<'info, Assignment>,
    // #[account(mut)]
    // pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut)]
	pub tasks: Account<'info, Tasks>,
	#[account(mut)]
	pub models: Account<'info, Models>,
	// #[account(mut, seeds = [b"referrer", creator.to_bytes().as_ref()], bump)]
	// pub referrer: Account<'info, Referrer>,
    #[account(mut)]
    pub miners_of_model: Account<'info, MinersOfModel>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"vault", sol_learn_account.key().as_ref()], 
        bump = vault_wallet_owner_pda.bump,
    )]
	pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(inference_id: u64)]
pub struct UpdateInferVld<'info> {
    #[account(mut)]
    pub infs: Account<'info, Inference>,
    pub signer: Signer<'info>,
    #[account(mut)]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VotingInfo {
    pub total_commit: u8,
    pub total_reveal: u8,
}
pub struct Bytes32 {}
pub type Bytes32Set = Vec<Bytes32>;

#[account]
pub struct Boost {
    pub miner_timestamp: u64,
    pub validator_timestamp: u64,
    pub reserved1: u64,
    pub reserved2: u64,
}

pub enum DAOTokenReceiverRole {
    Miner,
    Validator,
    User,
    Referrer,
    Referee,
    L2Owner,
}

#[account]
pub struct DAOTokenReceiverInfo {
    pub receiver: Pubkey,
    pub amount: u64,
    pub role: u8,
}

#[account]
pub struct DAOTokenReceiverInfos {
    pub values: Vec<DAOTokenReceiverInfo>,
    pub bump: u8,
}

pub enum Vote {
    Nil,
    Disapproval,
    Approval,
}

// #[account]
// pub struct MinerInfo {
//     pub stake: u64,
//     pub commitment: [u8; 32],
//     pub address: Pubkey,
//     pub model_address: Pubkey,
//     pub last_claimed_epoch: u64,
//     pub active_time: u64,
//     pub tier: u16,
//     pub bump: u8,
// }

#[account]
pub struct Assignment {
    pub bump: u8,
    pub id: u64,
    pub inference_id: u64,
    pub commitment: [u8; 32],
    pub digest: [u8; 32],
    pub reveal_nonce: u64,
    pub worker: Pubkey,
    pub role: u8,
    pub vote: u8,
    pub output: Vec<u8>,
}

#[account]
pub struct MinerEpochState {
    pub perf_reward: u64,
    pub epoch_reward: u64,
    pub total_task_completed: u64,
    pub total_miner: u64,
    pub bump: u8,
}

#[account]
pub struct Hashes {
    pub values: Vec<[u8; 32]>,
}

#[account]
pub struct DAOTokenPercentage {
    pub miner_percentage: u16,
    pub user_percentage: u16,
    pub referrer_percentage: u16,
    pub referee_percentage: u16,
    pub l2_owner_percentage: u16,
}

#[derive(Accounts)]
#[instruction(epoch_id: u64)]
pub struct UpdateEpochVld<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    // #[account(mut)]
    // pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut, seeds = [b"reward_in_epoch", epoch_id.to_le_bytes().as_ref()], bump = miner_reward.bump)]
    pub miner_reward: Account<'info, MinerEpochState>,
    // #[account(mut)]
    // pub miner: Account<'info, Worker>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SlashMinerByAdminVld<'info> {
    pub system_program: Program<'info, System>,
    // #[account(mut)]
    // pub sol_learn_account: Account<'info, SolLearnInfo>,
    // #[account(mut)]
    // pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut)]
    pub miner_reward: Account<'info, MinerEpochState>,
    #[account(mut)]
    pub miner: Account<'info, MinerInfo>,
    #[account(mut)]
    pub miners_of_model: Account<'info, MinersOfModel>,
    /// CHECK:
    #[account(mut, constraint = sol_learn_account.admin == signer.key())]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(assignment_id: u64)]
pub struct CreateAssignmentVld<'info> {
	#[account(init,
        payer = signer,
        space = 8*3 + 32*3 + 1*3 + 8 + 8,
        seeds = [b"assignment", assignment_id.to_le_bytes().as_ref()],
        bump
    )]
	pub assignment: Account<'info, Assignment>,
	#[account(mut)]
	pub tasks: Account<'info, Tasks>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(assignment_id: u64)]
pub struct PayMinerVld<'info> {
	#[account(mut)]
	pub tasks: Account<'info, Tasks>,
	#[account(mut,
        seeds = [b"assignment", assignment_id.to_le_bytes().as_ref()],
        bump = assignment.bump
    )]
    pub assignment: Account<'info, Assignment>,
	#[account(mut)]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub token_recipient: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(assignment_id: u64)]
pub struct SlashMinerVld<'info> {
	// pub system_program: Program<'info, System>,
	#[account(mut)]
	pub sol_learn_account: Account<'info, SolLearnInfo>,
	// #[account(mut)]
	// pub miner_addresses: Account<'info, Pubkeys>,
	#[account(mut)]
	pub miner: Account<'info, MinerInfo>,
	#[account(mut)]
	pub tasks: Account<'info, Tasks>,
	#[account(mut)]
    pub assignment: Account<'info, Assignment>,
    #[account(mut)]
    pub miners_of_model: Account<'info, MinersOfModel>,
	// #[account(mut)]
	// pub signer: Signer<'info>,
	#[account(mut)]
	pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
	#[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
	#[account(mut)]
    pub token_recipient: InterfaceAccount<'info, TokenAccount>,
}



#[derive(Accounts)]
#[instruction(assignment_id: u64, inference_id: u64)]
pub struct UpdateAssignmentVld<'info> {
    #[account(mut)]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    #[account(mut)]
    pub infs: Account<'info, Inference>,
    #[account(mut)]
    pub assignment: Account<'info, Assignment>,
    // #[account(mut)]
    // pub miner_addresses: Account<'info, Pubkeys>,
    // #[account(mut)]
    // pub miner_reward: Account<'info, MinerEpochState>,
    #[account(mut)]
    pub miner: Account<'info, MinerInfo>,
    #[account(init_if_needed, payer = signer, space = 24, seeds = [b"voting_info", inference_id.to_le_bytes().as_ref()], bump )]
    pub voting_info: Account<'info, VotingInfo>,
    #[account(mut)]
	pub tasks: Account<'info, Tasks>,
    #[account(init_if_needed, payer = signer, space = 1024, seeds = [b"dao_receivers_infos", sol_learn_account.key().as_ref()], bump)]
    pub dao_receiver_infos: Account<'info, DAOTokenReceiverInfos>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
    #[account(mut, constraint = vault_staking_wallet.owner == vault_wallet_owner_pda.key())]
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub token_recipient: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMinerAddressesByModelVld<'info> {
    #[account(mut)]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    pub signer: Signer<'info>,
    // #[account(mut)]
    // pub miner_addresses: Account<'info, Pubkeys>,
}

// #[derive(Accounts)]
// pub struct UpdateTaskVld<'info> {
// 	#[account(mut)]
// 	pub sol_learn_account: Account<'info, SolLearnInfo>,
// 	#[account(mut)]
// 	pub tasks: Account<'info, Tasks>,
// 	#[account(mut)]
// 	pub signer: Signer<'info>,
// }


// #[account]
// pub struct WorkerHubStorage {
//     // pub models: Vec<Models>,
//     pub miner_addresses: Pubkeys,
//     pub inference_number: u64,
//     pub assignment_number: u64,
//     pub miner_min_stake: u64,
//     pub l2_owner: Pubkey,
//     pub treasury: Pubkey,
//     pub fee_l2_percentage: u16,
//     pub fee_treasury_percentage: u16,
//     pub fee_ratio_miner_validator: u16,
//     pub submit_duration: u64,
//     pub commit_duration: u64,
//     pub reveal_duration: u64,
//     pub penalty_duration: u64,
//     pub miner_requirement: u8,
//     pub blocks_per_epoch: u64,
//     pub last_block: u64,
//     pub fine_percentage: u16,
//     pub dao_token: Pubkey,
//     pub dao_token_reward: u64,
//     pub dao_token_percentage: DAOTokenPercentage,

//     // pub current_epoch: u64,
//     // pub reward_per_epoch: u64,
//     // pub min_fee_to_use: u64,
//     // pub randomizer: Pubkey,
//     // pub miners: HashMap<Pubkey, Worker>,
//     // pub minerAddressesByModel: HashMap<Pubkey, Pubkeys>,
//     // pub modelAddresses: Pubkeys,
//     // pub minerUnstakeRequests: HashMap<Pubkey, UnstakeRequest>,
//     // pub inferences: HashMap<u64, Inference>,
//     // pub assignments: HashMap<u64, Assignment>,
//     // pub votingInfo: HashMap<u64, VotingInfo>,
//     // pub digests: HashMap<u64, Bytes32Set>,
//     // pub countDigest: HashMap<Bytes32, u8>,
//     // pub assignmentsByMiner: HashMap<Pubkey, Vec<u64>>,
//     // pub assignmentsByInference: HashMap<u64, Vec<u64>>,
//     // pub unstakeDelayTime: u64,
//     // pub maximumTier: u16,
//     // pub rewardPerEpoch: u64,
//     // pub minerRewards: HashMap<Pubkey, u64>,
//     // pub boost: HashMap<Pubkey, Boost>,
//     // pub isReferrer: HashMap<Pubkey, bool>,
//     // pub referrerOf: HashMap<Pubkey, Pubkey>,
//     // pub workerHubScoring: Pubkey,
//     // pub modelScoring: Pubkey,
//     // pub daoReceiversInfo: HashMap<u64, Vec<DAOTokenReceiverInfo>>,
//     // pub wEAI: Pubkey,
// }

// pub type WorkerHubStorage = SolLearnInfo;

#[derive(PartialEq)]
pub enum FnType {
	CreateAssignment,
	PayMiner,
	SlashMiner,
}

#[account]
pub struct Task {
    _b: [u8; 50],
}

impl Task {
    pub fn fn_type(&self) -> FnType {
        match self._b[0] {
            0 => FnType::CreateAssignment,
            1 => FnType::PayMiner,
            2 => FnType::SlashMiner,
            _ => panic!("Invalid task"),
        }
    }

    pub fn data(&self) -> Vec<u8> {
        let mut data = [0; 49];
        data.copy_from_slice(&self._b[1..]);
        data.to_vec()
    }

    pub fn new(fn_type: FnType, data: Vec<u8>) -> Self {
        let mut task = Task {
            _b: [0; 50],
        };
        task._b[0] = match fn_type {
            FnType::CreateAssignment => 0,
            FnType::PayMiner => 1,
            FnType::SlashMiner => 2,
        };
        task._b[1..].copy_from_slice(&data);
        task
    }
}


#[account]
pub struct Tasks {
    pub bump: u8,
	pub values: Vec<u8>,
}

impl Tasks {
    pub fn push_task(&mut self, task: Task) {
        self.values.extend(task._b.to_vec());
    }

    pub fn pop_task(&mut self) -> Option<Task> {
        if self.values.len() < 50 {
            return None;
        }
        let task: Vec<u8> = self.values.drain(self.values.len() - 50..).collect();
        
        Some(Task {
            _b: task.try_into().unwrap(),
        })
    }
}


#[event]
pub struct NewInference {
    pub inference_id: u64,
    pub model_address: Pubkey,
    pub creator: Pubkey,
    pub value: u64,
}

#[event]
pub struct TopUpInfer {
    pub inference_id: u64,
    pub creator: Pubkey,
    pub value: u64,
}

#[event]
pub struct NewAssignment {
    pub assignment_id: u64,
    pub inference_id: u64,
    pub worker: Pubkey,
    // pub expired_at: u64,
}

#[event]
pub struct MinerRoleSeized {
    pub assignment_id: u64,
    pub inference_id: u64,
    pub sender: Pubkey,
}

#[event]
pub struct InferenceStatusUpdate {
    pub inference_id: u64,
    pub status: u8,
}

#[event]
pub struct SolutionSubmission {
    pub sender: Pubkey,
    pub assignment_id: u64,
}

#[event]
pub struct CommitmentSubmission {
    pub sender: Pubkey,
    pub assignment_id: u64,
    pub commitment: [u8; 32],
}

#[event]
pub struct RevealSubmission {
    pub sender: Pubkey,
    pub assignment_id: u64,
    pub nonce: u64,
    pub data: Vec<u8>,
}

#[event]
pub struct MinerPenalized {
    pub miner: Pubkey,
    pub model_address: Pubkey,
    pub treasury: Pubkey,
    pub fine: u64,
}

#[event]
pub struct MinerDeactivated {
    pub miner: Pubkey,
    pub model_address: Pubkey,
    pub active_time: u64,
}

#[event]
pub struct FinePercentageUpdated {
    // pub fine_percentage: u16,
    pub new_fine_percentage: u16,
}

#[event]
pub struct PenaltyDurationUpdated {
    // pub penalty_duration: u64,
    pub new_penalty_duration: u64,
}

#[event]
pub struct MinFeeToUseUpdated {
    // pub min_fee_to_use: u64,
    pub new_min_fee_to_use: u64,
}

#[event]
pub struct L2OwnerUpdated {
    // pub l2_owner: Pubkey,
    pub new_l2_owner: Pubkey,
}

#[event]
pub struct DaoTokenUpdated {
    // pub dao_token: Pubkey,
    pub new_dao_token: Pubkey,
}

#[event]
pub struct TreasuryAddressUpdated {
    // pub treasury: Pubkey,
    pub new_treasury: Pubkey,
}

#[event]
pub struct FeeRatioMinerValidatorUpdated {
    pub new_fee_ratio_miner_validator: u64,
}

#[event]
pub struct DaoTokenRewardUpdated {
    pub new_dao_token_reward: u64,
}



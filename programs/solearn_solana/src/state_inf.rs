use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{MinerInfo, Models, SolLearnInfo, VaultAccount};

#[derive(Accounts)]
pub struct UpdateParamsVld<'info> {
    #[account(mut)]
    pub wh_account: Account<'info, WorkerHubStorage>,
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
    pub wh_account: Account<'info, WorkerHubStorage>,
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
    pub bump: u8,
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
        space = 8 + 2 + 4 + 2000 + 1, seeds = [b"inference", inference_id.to_le_bytes().as_ref()], bump
    )]
    pub infs: Account<'info, Inference>,
    pub system_program: Program<'info, System>,
    #[account(init, payer = signer, space = 8 + 8)]
    pub wh_account: Account<'info, WorkerHubStorage>,
    #[account(mut, seeds = [b"assignment", signer.key().as_ref()], bump = assignment.bump)]
    pub assignment: Account<'info, Assignment>,
    #[account(mut)]
    pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut)]
	pub tasks: Account<'info, Tasks>,
	#[account(mut)]
	pub models: Account<'info, Models>,
	#[account(mut, seeds = [b"referrer", creator.to_bytes().as_ref()], bump = referrer.bump)]
	pub referrer: Account<'info, Referrer>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
	pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
}

#[derive(Accounts)]
#[instruction(inference_id: u64)]
pub struct UpdateInferVld<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub infs: Account<'info, Inference>,
    pub signer: Signer<'info>,
    #[account(mut)]
	pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
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
    pub id: u64,
    pub inference_id: u64,
    pub commitment: [u8; 32],
    pub digest: [u8; 32],
    pub reveal_nonce: u64,
    pub worker: Pubkey,
    pub role: u8,
    pub vote: u8,
    pub output: Vec<u8>,
    pub bump: u8,
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
    pub wh_account: Account<'info, WorkerHubStorage>,
    #[account(mut)]
    pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut, seeds = [b"reward_in_epoch", epoch_id.to_le_bytes().as_ref()], bump = miner_reward.bump)]
    pub miner_reward: Account<'info, MinerEpochState>,
    // #[account(mut)]
    // pub miner: Account<'info, Worker>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct SlashMinerByAdminVld<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub wh_account: Account<'info, WorkerHubStorage>,
    #[account(mut)]
    pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut)]
    pub miner_reward: Account<'info, MinerEpochState>,
    #[account(mut)]
    pub miner: Account<'info, MinerInfo>,
    /// CHECK:
    #[account(mut, constraint = sol_learn_account.admin == signer.key())]
    pub sol_learn_account: Account<'info, SolLearnInfo>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(assignment_id: u64)]
pub struct CreateAssignmentVld<'info> {
	#[account(mut)]
	pub assignment: Account<'info, Assignment>,
	#[account(mut)]
	pub tasks: Account<'info, Tasks>,
}

#[derive(Accounts)]
#[instruction(assignment_id: u64)]
pub struct PayMinerVld<'info> {
	pub system_program: Program<'info, System>,
	#[account(mut)]
	pub tasks: Account<'info, Tasks>,
	#[account(mut)]
    pub assignment: Account<'info, Assignment>,
    /// CHECK
	#[account(mut)]
	pub recipient: UncheckedAccount<'info>,
	#[account(mut)]
	pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
}

#[derive(Accounts)]
#[instruction(assignment_id: u64)]
pub struct SlashMinerVld<'info> {
	// pub system_program: Program<'info, System>,
	#[account(mut)]
	pub wh_account: Account<'info, WorkerHubStorage>,
	#[account(mut)]
	pub miner_addresses: Account<'info, Pubkeys>,
	#[account(mut)]
	pub miner: Account<'info, MinerInfo>,
	#[account(mut)]
	pub tasks: Account<'info, Tasks>,
	#[account(mut)]
    pub assignment: Account<'info, Assignment>,
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
#[instruction(assignment_id: u64)]
pub struct UpdateAssignmentVld<'info> {
	pub system_program: Program<'info, System>,
    #[account(mut)]
    pub wh_account: Account<'info, WorkerHubStorage>,
    #[account(mut)]
    pub infs: Account<'info, Inference>,
    #[account(mut)]
    pub assignment: Account<'info, Assignment>,
    #[account(mut)]
    pub miner_addresses: Account<'info, Pubkeys>,
    // #[account(mut)]
    // pub miner_reward: Account<'info, MinerEpochState>,
    #[account(mut)]
    pub miner: Account<'info, MinerInfo>,
    #[account(mut)]
    pub voting_info: Account<'info, VotingInfo>,
    #[account(mut)]
	pub tasks: Account<'info, Tasks>,
    #[account(mut)]
	pub vault_wallet_owner_pda: Account<'info, VaultAccount>,
	/// CHECK
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"dao_receivers_info", signer.key().as_ref()], bump = dao_receiver_infos.bump)]
    pub dao_receiver_infos: Account<'info, DAOTokenReceiverInfos>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateMinerAddressesByModelVld<'info> {
    #[account(mut)]
    pub wh_account: Account<'info, WorkerHubStorage>,
    pub signer: Signer<'info>,
    #[account(mut)]
    pub miner_addresses: Account<'info, Pubkeys>,
}

#[derive(Accounts)]
pub struct UpdateTaskVld<'info> {
	#[account(mut)]
	pub wh_account: Account<'info, WorkerHubStorage>,
	#[account(mut)]
	pub tasks: Account<'info, Tasks>,
	#[account(mut)]
	pub signer: Signer<'info>,
}


#[account]
pub struct WorkerHubStorage {
    // pub models: Vec<Models>,
    pub miner_addresses: Pubkeys,
    pub inference_number: u64,
    pub assignment_number: u64,
    pub miner_minimum_stake: u64,
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
    pub current_epoch: u64,
    pub reward_per_epoch: u64,
    pub fine_percentage: u16,
    pub dao_token: Pubkey,
    pub dao_token_reward: u64,
    pub dao_token_percentage: DAOTokenPercentage,
    pub min_fee_to_use: u64,
    // pub randomizer: Pubkey,
    // pub miners: HashMap<Pubkey, Worker>,
    // pub minerAddressesByModel: HashMap<Pubkey, Pubkeys>,
    // pub modelAddresses: Pubkeys,
    // pub minerUnstakeRequests: HashMap<Pubkey, UnstakeRequest>,
    // pub inferences: HashMap<u64, Inference>,
    // pub assignments: HashMap<u64, Assignment>,
    // pub votingInfo: HashMap<u64, VotingInfo>,
    // pub digests: HashMap<u64, Bytes32Set>,
    // pub countDigest: HashMap<Bytes32, u8>,
    // pub assignmentsByMiner: HashMap<Pubkey, Vec<u64>>,
    // pub assignmentsByInference: HashMap<u64, Vec<u64>>,
    // pub unstakeDelayTime: u64,
    // pub maximumTier: u16,
    // pub rewardPerEpoch: u64,
    // pub minerRewards: HashMap<Pubkey, u64>,
    // pub boost: HashMap<Pubkey, Boost>,
    // pub isReferrer: HashMap<Pubkey, bool>,
    // pub referrerOf: HashMap<Pubkey, Pubkey>,
    // pub workerHubScoring: Pubkey,
    // pub modelScoring: Pubkey,
    // pub daoReceiversInfo: HashMap<u64, Vec<DAOTokenReceiverInfo>>,
    // pub wEAI: Pubkey,
}

impl WorkerHubStorage {
    pub const INIT_LEN: usize = 8 + 8 + 8 + 8 + 32 + 32 + 2 + 2 + 2 + 8 + 8 + 8 + 8
        + 1 + 8 + 8 + 8 + 2 + 32 + 8
        + 2 * 5
        + 8 + 8;
    pub fn len(&self) -> usize {
        8 + 8 + 8 + 8 + 32 + 32 + 2 + 2 + 2 + 8 + 8 + 8 + 8
        + 1 + 8 + 8 + 8 + 2 + 32 + 8
        + 2 * 5
        + 8 + self.miner_addresses.len()
    }
}

pub enum FnType {
	CreateAssignment,
	PayMiner,
	SlashMiner,
}

#[account]
pub struct Task {
	pub fn_type: u8,
	pub data: Vec<u8>,
}

#[account]
pub struct Tasks {
	pub values: Vec<Task>,
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



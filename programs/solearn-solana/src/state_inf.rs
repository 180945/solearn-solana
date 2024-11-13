use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateParamsVld<'info> {
    #[account(init, payer = user, space = 8 + 8)]
    pub wh_account: Account<'info, WorkerHubStorage>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Pubkeys {
    pub values: Vec<Pubkey>,
}

#[account]
pub struct Model {
    pub minimum_fee: u64,
    pub tier: u32,
    pub address: Pubkey,
}
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

#[derive(Accounts)]
pub struct InferVld<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 2 + 4 + 2000 + 1, seeds = [b"inference", signer.key().as_ref()], bump
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
    pub signer: Signer<'info>,
    /// CHECK:
    #[account(mut, constraint = *__program_id == recipient.key())]
    pub recipient: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateInferVld<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut, seeds = [b"inference", signer.key().as_ref()], bump = infs.bump)]
    pub infs: Account<'info, Inference>,
    pub signer: Signer<'info>,
    /// CHECK:
    #[account(mut, constraint = *__program_id == recipient.key())]
    pub recipient: UncheckedAccount<'info>,
}

#[account]
pub struct VotingInfo {
    pub total_commit: u8,
    pub total_reveal: u8,
}
pub struct Bytes32 {}
pub type Bytes32Set = Vec<Bytes32>;
pub struct Boost {}
pub struct DAOTokenReceiverInfo {}

#[account]
pub struct Worker {
    pub stake: u64,
    pub commitment: [u8; 32],
    pub address: Pubkey,
    pub model_address: Pubkey,
    pub last_claimed_epoch: u64,
    pub active_time: u64,
    pub tier: u16,
    pub bump: u8,
}

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

#[derive(Accounts)]
pub struct UpdateEpochVld<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub wh_account: Account<'info, WorkerHubStorage>,
    #[account(mut)]
    pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut, seeds = [b"reward_in_epoch", signer.key().as_ref()], bump = miner_reward.bump)]
    pub miner_reward: Account<'info, MinerEpochState>,
    #[account(mut)]
    pub miner: Account<'info, Worker>,
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateAssignmentVld<'info> {
    #[account(mut)]
    pub wh_account: Account<'info, WorkerHubStorage>,
    #[account(mut)]
    pub infs: Account<'info, Inference>,
    #[account(mut)]
    pub assignment: Account<'info, Assignment>,
    #[account(mut)]
    pub miner_addresses: Account<'info, Pubkeys>,
    #[account(mut)]
    pub miner_reward: Account<'info, MinerEpochState>,
    #[account(mut)]
    pub miner: Account<'info, Worker>,
    #[account(mut)]
    pub voting_info: Account<'info, VotingInfo>,
    #[account(mut)]
    pub digests: Account<'info, Hashes>,
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

#[account]
pub struct WorkerHubStorage {
    // pub randomizer: Pubkey,
    pub models: Vec<Model>,
    // pub miners: HashMap<Pubkey, Worker>,
    // pub minerAddressesByModel: HashMap<Pubkey, Pubkeys>,
    // pub modelAddresses: Pubkeys,
    pub miner_addresses: Pubkeys,
    // pub minerUnstakeRequests: HashMap<Pubkey, UnstakeRequest>,
    pub inference_number: u64,
    // pub inferences: HashMap<u64, Inference>,
    pub assignment_number: u64,
    // pub assignments: HashMap<u64, Assignment>,
    // pub votingInfo: HashMap<u64, VotingInfo>,
    // pub digests: HashMap<u64, Bytes32Set>,
    // pub countDigest: HashMap<Bytes32, u8>,
    // pub assignmentsByMiner: HashMap<Pubkey, Vec<u64>>,
    // pub assignmentsByInference: HashMap<u64, Vec<u64>>,
    pub miner_minimum_stake: u64,
    // pub l2Owner: Pubkey,
    // pub treasury: Pubkey,
    pub fee_l2_percentage: u16,
    pub fee_treasury_percentage: u16,
    // pub feeRatioMinerValidator: u16,
    pub submit_duration: u64,
    pub commit_duration: u64,
    pub reveal_duration: u64,
    pub penalty_duration: u64,
    // pub unstakeDelayTime: u64,
    pub miner_requirement: u8,
    // pub maximumTier: u16,
    pub blocks_per_epoch: u64,
    pub last_block: u64,
    pub current_epoch: u64,
    pub reward_per_epoch: u64,
    // pub rewardPerEpoch: u64,
    pub fine_percentage: u16,
    // pub minerRewards: HashMap<Pubkey, u64>,
    // pub boost: HashMap<Pubkey, Boost>,
    // pub isReferrer: HashMap<Pubkey, bool>,
    // pub daoToken: Pubkey,
    // pub daoTokenReward: u64,
    // pub daoTokenPercentage: Pubkey,
    // pub referrerOf: HashMap<Pubkey, Pubkey>,
    // pub minFeeToUse: u64,
    // pub workerHubScoring: Pubkey,
    // pub modelScoring: Pubkey,
    // pub daoReceiversInfo: HashMap<u64, Vec<DAOTokenReceiverInfo>>,
    // pub wEAI: Pubkey,
}
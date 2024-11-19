use anchor_lang::prelude::*;

#[error_code]
pub enum SolLearnError {
    MustGreatThanMinStake,
    NoModelRegistered,
    NotAcitveYet,
    Joined,
    Activated,
    MustJoinMintingFirst,
    MinerNotRegistered,
    Unstaked,
    MinerNotActive,
    MustUseJoinMintingInstead,
    NeedToWait,
    UnstakeNotInitYet, 
    CanNotClaim,
    NothingToClaim,
    ModelNotExist,
    NoEnoughVault,
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Fee too low")]
    FeeTooLow,
    #[msg("Zero value")]
    ZeroValue,
    #[msg("Inference must be in solving state")]
    InferMustBeSolvingState,
    #[msg("Wrong recipient")]
    WrongRecipient,
    #[msg("Wrong sender")]
    WrongSender,
    #[msg("This inference has been seized")]
    InferenceSeized,
    #[msg("Invalid reveal")]
    InvalidReveal,
    #[msg("Invalid epoch id")]
    InvalidEpochId,
    #[msg("Wrong inference id")]
    WrongInferenceId,
    #[msg("Wrong assignment id")]
    WrongAssignmentId,
    #[msg("No miner available")]
    NoMinerAvailable,
    #[msg("Must execute pending tasks first")]
    MustWaitTasks,
    #[msg("No valid task")]
    NoValidTask,
    #[msg("Epoch reward up-to-date")]
    EpochRewardUpToDate,
    #[msg("Insufficient funds")]
    InsufficientFunds,
}

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
    #[msg("This inference has been seized")]
    InferenceSeized,
    #[msg("Invalid reveal")]
    InvalidReveal,
}

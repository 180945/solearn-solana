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
}
use anchor_lang::prelude::*;
use solearn_solana::program::Solearn;

// init pda to store list of models
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub model_storage: Account<'info, ModelStorage>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateParamsVld<'info> {
    #[account(mut)]
    pub model_storage: Account<'info, ModelStorage>,
    #[account(mut, constraint = model_storage.admin == *admin.key)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct CpiInferVld<'info> {
    // #[account(mut)]
    // pub model_storage: Account<'info, ModelStorage>,
    pub callee: Program<'info, Solearn>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct ModelStorage {
    pub identifier: u64,
    pub name: String,
    pub metadata: String,
    pub worker_hub: Pubkey,
    pub model_collection: Pubkey,
    pub admin: Pubkey,
    pub bump: u8,
}

#[event]
pub struct WorkerHubUpdate {
    pub new_worker_hub: Pubkey,
}

#[event]
pub struct IdentifierUpdate {
    pub new_identifier: u64,
}

#[event]
pub struct NameUpdate {
    pub new_name: String,
}

#[event]
pub struct MetadataUpdate {
    pub new_metadata: String,
}

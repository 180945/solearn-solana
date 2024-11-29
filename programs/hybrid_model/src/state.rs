use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use solearn_solana::program::Solearn;

// init pda to store list of models
#[derive(Accounts)]
#[instruction(identifier: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        space = ModelStorage::LEN,
        payer = admin,
        seeds = ["model_storage".as_bytes(), identifier.to_le_bytes().as_ref()],
        bump,
    )]
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
#[instruction(id_collection: u64, id_nft: u64)]
pub struct CpiInferVld<'info> {
    /// CHECK:
    #[account(mut)]
    pub infs: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK:
    #[account(mut)]
    pub sol_learn_account: UncheckedAccount<'info>,
    /// CHECK:
    // #[account(mut)]
    // pub assignment: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub miners_of_model: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub tasks: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub vault_wallet_owner_pda: UncheckedAccount<'info>,
    pub solearn_program: Program<'info, Solearn>,
    #[account( 
        seeds = ["mint".as_bytes(), 
                id_collection.to_le_bytes().as_ref(),
                id_nft.to_le_bytes().as_ref()], 
        bump,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub agent_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub inferer_token_account: InterfaceAccount<'info, TokenAccount>,
    pub miner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    pub vault_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    /// CHECK:
    #[account(mut)]
    pub models: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub referrer: UncheckedAccount<'info>,
    /// init new promt account
    // #[account( 
    //     seeds = ["promt".as_bytes(), 
    //             id_collection.to_le_bytes().as_ref(),
    //             id_nft.to_le_bytes().as_ref(),
    //             agent_token_account.key().as_ref()], 
    //     bump = promt_account.bump,
    // )]
    // pub promt_account: Account<'info, PromptAccount>,
    pub token_program: Interface<'info, TokenInterface>,
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

impl ModelStorage {
    pub const LEN: usize = 8 + 64 + 256 + 32 + 32 + 32 + 1;
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

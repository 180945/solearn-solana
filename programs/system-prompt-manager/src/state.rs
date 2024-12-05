use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::token::Token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::Metadata;
use solearn_solana::program::Solearn;

// init new nft
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct CreateNFT<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,  
    /// CHECK:  
    #[account( 
        init,
        payer = payer, 
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority,
        seeds = ["mint".as_bytes(), id.to_le_bytes().as_ref()], 
        bump,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    /// CHECK:
    pub master_edition_account: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    /// CHECK:
    pub nft_metadata: UncheckedAccount<'info>,
}

#[derive(Accounts)]
#[instruction(id_collection: u64, id_nft: u64)]
pub struct MintToCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account( 
        init,
        payer = authority, 
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority,
        seeds = ["mint".as_bytes(), 
                id_collection.to_le_bytes().as_ref(),
                id_nft.to_le_bytes().as_ref()], 
        bump,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    /// CHECK:
    pub master_edition_account: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    /// CHECK:
    pub nft_metadata: UncheckedAccount<'info>,
    #[account( 
        seeds = ["mint".as_bytes(), id_collection.to_le_bytes().as_ref()], 
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,
    // #[account( 
    //     associated_token::mint = mint,
    //     associated_token::authority = authority,
    // )]
    pub collection: InterfaceAccount<'info, TokenAccount>,
}

#[derive(Accounts)]
#[instruction(id_collection: u64, id_nft: u64, prompt: String)]
pub struct AddPrompt<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    /// init new prompt account
    #[account( 
        init,
        payer = payer, 
        seeds = ["prompt".as_bytes(), 
                id_collection.to_le_bytes().as_ref(),
                id_nft.to_le_bytes().as_ref(),
                token_account.key().as_ref()],
        space = 8 + 1 + 8 + 4 + prompt.len(),
        bump,
    )]
    pub prompt_account: Account<'info, PromptAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id_collection: u64, id_nft: u64, prompt: String)]
pub struct UpdatePrompt<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    #[account( 
        mut,
        realloc = 8 + 1 + 8 + 4 + prompt.len(),
        realloc::payer = payer,
        realloc::zero = false,
        seeds = ["prompt".as_bytes(), 
                id_collection.to_le_bytes().as_ref(),
                id_nft.to_le_bytes().as_ref(),
                token_account.key().as_ref()], 
        bump,
    )]
    pub prompt_account: Account<'info, PromptAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id_collection: u64, id_nft: u64, prompt: String)]
pub struct UpdateFee<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account( 
        seeds = ["mint".as_bytes(), 
                id_collection.to_le_bytes().as_ref(),
                id_nft.to_le_bytes().as_ref()], 
        bump,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = authority,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    /// init new prompt account
    #[account( 
        mut,
        seeds = ["prompt".as_bytes(), 
                id_collection.to_le_bytes().as_ref(),
                id_nft.to_le_bytes().as_ref(),
                token_account.key().as_ref()], 
        bump = prompt_account.bump,
    )]
    pub prompt_account: Account<'info, PromptAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id_collection: u64, id_nft: u64)]
pub struct SytemInfer<'info> {
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
    // #[account(mut)]
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
    #[account( 
        seeds = ["promt".as_bytes(), 
                id_collection.to_le_bytes().as_ref(),
                id_nft.to_le_bytes().as_ref(),
                agent_token_account.key().as_ref()], 
        bump = promt_account.bump,
    )]
    pub promt_account: Account<'info, PromptAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}


/// STRUCTS
#[account]
pub struct PromptAccount {
    pub bump: u8,
    pub fee: u64, // fee per request
    pub data: Vec<u8>, 
}


/// EVENTS
#[event]
pub struct PromptUpdated {
    pub id_collection: u64,
    pub id_nft: u64,
    pub prompt: Vec<u8>,
}


#[event]
pub struct FeeUpdated {
    pub id_collection: u64,
    pub id_nft: u64,
    pub fee: u64,
}


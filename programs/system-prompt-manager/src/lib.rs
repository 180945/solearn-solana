pub mod errors;
pub mod state;

use state::*;
use errors::*;

use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, MintTo};
use mpl_token_metadata::types::{Collection, Creator, DataV2};
use anchor_spl::metadata::{ 
    create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3, CreateMetadataAccountsV3
};
use solearn_solana::cpi::accounts::InferVld;

declare_id!("nuvdhmYq5Z2Eg4nBi29Tu2VcbpE9nuiCQ68rkyAB3A1");

#[program]
pub mod nft_program {
    use super::*;

    pub fn create_single_nft(
        ctx: Context<CreateNFT>,
        id: u64,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        msg!("Creating seeds");
        let id_bytes = id.to_le_bytes();
        let seeds = &["mint".as_bytes(),id_bytes.as_ref(),&[ctx.bumps.mint],];

        msg!("Run mint_to");

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &[&seeds[..]],
            ),
            1, // 1 token
        )?;

        msg!("Run create metadata accounts v3");

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    metadata: ctx.accounts.nft_metadata.to_account_info(),
                    mint_authority: ctx.accounts.authority.to_account_info(),
                    update_authority: ctx.accounts.authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&seeds[..]],
            ),
            DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;

        msg!("Run create master edition v3");

        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    edition: ctx.accounts.master_edition_account.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    metadata: ctx.accounts.nft_metadata.to_account_info(),
                    mint_authority: ctx.accounts.authority.to_account_info(),
                    update_authority: ctx.accounts.authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&seeds[..]],
            ),
            Some(1),
        )?;

        msg!("Minted NFT successfully");

        Ok(())
    }

    pub fn mint_to_collection(
        ctx: Context<MintToCollection>,
        id_collection: u64,
        id_nft: u64,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        msg!("Creating seeds");
        let id_bytes = id_collection.to_le_bytes();
        let id_nft_bytes = id_nft.to_le_bytes();
        let seeds = &[
            "mint".as_bytes(),
            id_bytes.as_ref(),
            id_nft_bytes.as_ref(),
            &[ctx.bumps.mint],
        ];

        msg!("Run mint_to");

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &[&seeds[..]],
            ),
            1, // 1 token
        )?;

        msg!("Run create metadata accounts v3");

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    metadata: ctx.accounts.nft_metadata.to_account_info(),
                    mint_authority: ctx.accounts.authority.to_account_info(),
                    update_authority: ctx.accounts.authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&seeds[..]],
            ),
            DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: Some(vec![Creator {
                    address: ctx.accounts.payer.key(),
                    verified: true,
                    share: 100,
                }]),
                collection: Some(Collection {
                    key: ctx.accounts.collection.key(),
                    verified: false,
                }),
                uses: None,
            },
            true,
            true,
            None,
        )?;

        msg!("Run create master edition v3");

        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    edition: ctx.accounts.master_edition_account.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    metadata: ctx.accounts.nft_metadata.to_account_info(),
                    mint_authority: ctx.accounts.authority.to_account_info(),
                    update_authority: ctx.accounts.authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&seeds[..]],
            ),
            Some(1),
        )?;

        msg!("Minted NFT successfully");

        Ok(())
    }

    pub fn add_prompt(ctx: Context<AddPrompt>, id_collection: u64, id_nft: u64, prompt: Vec<u8>) -> Result<()> {
        msg!("Instruction: Add Prompt");

        // Initialize the prompt account with the given prompt data
        ctx.accounts.promt_account.data = prompt.clone();

        emit!(PromptUpdated {
            id_collection,
            id_nft,
            prompt,
        });

        Ok(())
    }

    pub fn update_prompt(ctx: Context<UpdatePrompt>, id_collection: u64, id_nft: u64, prompt: Vec<u8>) -> Result<()> {
        msg!("Instruction: Update Prompt");

        // Update the prompt account with the new prompt data
        ctx.accounts.promt_account.data = prompt.clone();

        emit!(PromptUpdated {
            id_collection,
            id_nft,
            prompt,
        });

        Ok(())
    }

    pub fn update_fee(ctx: Context<UpdateFee>, id_collection: u64, id_nft: u64, fee: u64) -> Result<()> {
        msg!("Instruction: Update Fee");

        // Update the prompt account with the new fee
        ctx.accounts.promt_account.fee = fee;

        emit!(FeeUpdated {
            id_collection,
            id_nft,
            fee,
        });

        Ok(())
    }
    
    // infer 
    pub fn infer_request(ctx: Context<SytemInfer>, input: Vec<u8>, creator: Pubkey, _value: u64, inference_id: u64,) -> Result<()> {
        msg!("Instruction: Infer Request");

        // transfer fee to agent owner
        
        // append infer request

        // call infer to workerhub
        let cpi_program = ctx.accounts.solearn_program.to_account_info();
        let cpi_accounts = InferVld {
            infs: ctx.accounts.infs.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(), // Add missing field
            wh_account: ctx.accounts.wh_account.to_account_info(), // Add missing field
            assignment: ctx.accounts.assignment.to_account_info(), // Add missing field
            miner_addresses: ctx.accounts.miner_addresses.to_account_info(), // Add missing field
            tasks: ctx.accounts.tasks.to_account_info(), // Add missing field
            signer: ctx.accounts.signer.to_account_info(), // Add missing field
            vault_wallet_owner_pda: ctx.accounts.vault_wallet_owner_pda.to_account_info(), // Add missing field
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        solearn_solana::cpi::infer(cpi_ctx, input, creator, _value, inference_id)?;
        

        Ok(())
    }

}

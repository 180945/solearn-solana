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
use anchor_spl::token::{self, Transfer};

declare_id!("8CgzLBj4wq4pwKMv52BGnhaJLE22LEsv7obNTJtASNps");

#[event]
pub struct NewCollection {
    pub collection_id: u64,
}


#[event]
pub struct NftCreated {
    pub collection_id: u64,
    pub nft_id: u64,
}


#[program]
pub mod prompt_system_manager {
    use super::*;

    pub fn create_single_nft(
        ctx: Context<CreateNFT>,
        id: u64,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let id_bytes = id.to_le_bytes();
        let seeds = &["mint".as_bytes(),id_bytes.as_ref(),&[ctx.bumps.mint],];

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

        emit!(NewCollection {
            collection_id: id,
        });

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
        let id_bytes = id_collection.to_le_bytes();
        let id_nft_bytes = id_nft.to_le_bytes();

        let seeds = &[
            "mint".as_bytes(),
            id_bytes.as_ref(),
            id_nft_bytes.as_ref(),
            &[ctx.bumps.mint],
        ];

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

        emit!(NftCreated {
            collection_id: id_collection,
            nft_id: id_nft,
        });

        Ok(())
    }

    pub fn add_prompt(ctx: Context<AddPrompt>, id_collection: u64, id_nft: u64, prompt: Vec<u8>) -> Result<()> {
        msg!("Instruction: Add Prompt");

        // Initialize the prompt account with the given prompt data
        ctx.accounts.prompt_account.data = prompt.clone();
        ctx.accounts.prompt_account.bump = ctx.accounts.prompt_account.bump;

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
        ctx.accounts.prompt_account.data = prompt.clone();

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
        ctx.accounts.prompt_account.fee = fee;

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

        // append infer request
        let infer_data = ctx.accounts.promt_account.data.clone().into_iter().chain(input.clone().into_iter()).collect::<Vec<u8>>();

        // extract and transfer fee to agent owner
        if ctx.accounts.promt_account.fee > _value {
            return Err(SystemPromptManagerError::InsufficientFunds.into());
        }

        let infer_value = _value - ctx.accounts.promt_account.fee;

        let cpi_accounts = Transfer {
            from: ctx.accounts.inferer_token_account.to_account_info(),
            to: ctx.accounts.agent_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, ctx.accounts.promt_account.fee)?;

        // call infer to workerhub
        let cpi_program = ctx.accounts.solearn_program.to_account_info();
        let cpi_accounts = InferVld {
            infs: ctx.accounts.infs.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(), // Add missing field
            sol_learn_account: ctx.accounts.sol_learn_account.to_account_info(), // Add missing field
            // assignment: ctx.accounts.assignment.to_account_info(), // Add missing field
            // miner_addresses: ctx.accounts.miner_addresses.to_account_info(), // Add missing field
            tasks: ctx.accounts.tasks.to_account_info(), // Add missing field
            signer: ctx.accounts.signer.to_account_info(), // Add missing field
            vault_wallet_owner_pda: ctx.accounts.vault_wallet_owner_pda.to_account_info(), // Add missing field
            miner_staking_wallet: ctx.accounts.miner_staking_wallet.to_account_info(),
            models: ctx.accounts.models.to_account_info(),
            // referrer: ctx.accounts.referrer.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            vault_staking_wallet: ctx.accounts.vault_staking_wallet.to_account_info(),
            miners_of_model: ctx.accounts.miners_of_model.to_account_info(),
            dao_receiver_infos: ctx.accounts.miners_of_model.to_account_info(),
            voting_info: ctx.accounts.miners_of_model.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        solearn_solana::cpi::infer(cpi_ctx, inference_id, creator, infer_data, infer_value, ctx.accounts.models.key())?;
        

        Ok(())
    }

}

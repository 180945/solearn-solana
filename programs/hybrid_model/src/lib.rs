pub mod errors;
pub mod state;
mod utils;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hash;
use anchor_lang::system_program;
use anchor_spl::token::{self, transfer_checked, Transfer, TransferChecked};
use solearn_solana::cpi::accounts::InferVld;
use errors::*;
use state::*;
use utils::*;

declare_id!("GJDRVDToZqT6ZQZ74TreUqm4tvR8yYhUcMwYKMHucoen");

#[program]
pub mod hybrid_model {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>,
                        identifier: u64,
                        name: String,
                        metadata: String,
                        worker_hub: Pubkey,
                        model_collection: Pubkey) -> Result<()> {
        
        msg!("Instruction: Initialize");
        let model_storage = &mut ctx.accounts.model_storage;
        model_storage.identifier = identifier;
        model_storage.name = name;
        model_storage.metadata = metadata;
        model_storage.worker_hub = worker_hub;
        model_storage.model_collection = model_collection;
        model_storage.admin = *ctx.accounts.admin.key;
        model_storage.bump = 0;
        Ok(())
    }

    pub fn set_worker_hub(ctx: Context<UpdateParamsVld>, worker_hub: Pubkey) -> Result<()> {
        msg!("Instruction: Set Worker Hub");
        let model_storage = &mut ctx.accounts.model_storage;
        model_storage.worker_hub = worker_hub;
        emit!(WorkerHubUpdate {
            new_worker_hub: worker_hub,
        });

        Ok(())
    }

    pub fn set_identifier(ctx: Context<UpdateParamsVld>, identifier: u64) -> Result<()> {
        msg!("Instruction: Set Identifier");
        let model_storage = &mut ctx.accounts.model_storage;
        model_storage.identifier = identifier;
        emit!(IdentifierUpdate {
            new_identifier: identifier,
        });
        Ok(())
    }

    pub fn set_name(ctx: Context<UpdateParamsVld>, name: String) -> Result<()> {
        msg!("Instruction: Set Name");
        let model_storage = &mut ctx.accounts.model_storage;
        model_storage.name = name.clone();
        emit!(NameUpdate {
            new_name: name,
        });
        Ok(())
    }

    pub fn set_metadata(ctx: Context<UpdateParamsVld>, metadata: String) -> Result<()> {
        msg!("Instruction: Set Metadata");
        let model_storage = &mut ctx.accounts.model_storage;
        model_storage.metadata = metadata.clone();
        emit!(MetadataUpdate {
            new_metadata: metadata,
        });
        Ok(())
    }

    pub fn set_model_id_by_collection(ctx: Context<UpdateParamsVld>, identifier: u64) -> Result<()> {
        msg!("Instruction: Set Model ID by Collection");
        let model_storage = &mut ctx.accounts.model_storage;
        // compare signer with model collection
        let sender = ctx.accounts.admin.key;
        if sender != &model_storage.model_collection {
            return Err(CustomError::Unauthorized.into());
        }

        model_storage.identifier = identifier;
        emit!(IdentifierUpdate {
            new_identifier: identifier,
        });
        Ok(())
    }

    pub fn infer(ctx: Context<CpiInferVld>,
        id_collection: u64,
        id_nft: u64,
        input: Vec<u8>,
        creator: Pubkey,
        _value: u64,
        inference_id: u64,
    ) -> Result<()> {
        msg!("Instruction: Infer");
        let cpi_program = ctx.accounts.solearn_program.to_account_info();
        let cpi_accounts = InferVld {
            infs: ctx.accounts.infs.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(), // Add missing field
            sol_learn_account: ctx.accounts.sol_learn_account.to_account_info(), // Add missing field
            assignment: ctx.accounts.assignment.to_account_info(), // Add missing field
            miner_addresses: ctx.accounts.miner_addresses.to_account_info(), // Add missing field
            tasks: ctx.accounts.tasks.to_account_info(), // Add missing field
            signer: ctx.accounts.signer.to_account_info(), // Add missing field
            vault_wallet_owner_pda: ctx.accounts.vault_wallet_owner_pda.to_account_info(), // Add missing field
            miner_staking_wallet: ctx.accounts.miner_staking_wallet.to_account_info(),
            models: ctx.accounts.models.to_account_info(),
            referrer: ctx.accounts.referrer.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            vault_staking_wallet: ctx.accounts.vault_staking_wallet.to_account_info(),
            miners_of_model: ctx.accounts.miner_addresses.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        solearn_solana::cpi::infer(cpi_ctx, input, creator, _value, inference_id, ctx.accounts.models.key())?;
        Ok(())
    }
}

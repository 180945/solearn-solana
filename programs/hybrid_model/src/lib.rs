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

declare_id!("7MHr6ZPGTWZkRk6m52GfEWoMxSV7EoDjYyoXAYf3MBwS");

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
        input: Vec<u8>,
        creator: Pubkey,
        _value: u64,
        inference_id: u64,
    ) -> Result<()> {
        msg!("Instruction: Infer");
        let cpi_program = ctx.accounts.callee.to_account_info();
        let cpi_accounts = InferVld {
            infs: todo!(),
            system_program: ctx.accounts.system_program.to_account_info(),
            wh_account: todo!(),
            assignment: todo!(),
            miner_addresses: todo!(),
            tasks: todo!(),
            models: todo!(),
            referrer: todo!(),
            signer: todo!(),
            vault_wallet_owner_pda: todo!()
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        solearn_solana::cpi::infer(cpi_ctx, input, creator, _value, inference_id)?;
        Ok(())
    }
}

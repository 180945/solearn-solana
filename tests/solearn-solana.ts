import { randomBytes } from 'node:crypto';
import { describe, it } from 'node:test';
import * as anchor from '@coral-xyz/anchor';
import { BN, type Program } from '@coral-xyz/anchor';
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  type TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from '@solana/spl-token';
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, type TransactionInstruction } from '@solana/web3.js';
import { BankrunProvider } from 'anchor-bankrun';
import { assert } from 'chai';
import { startAnchor } from 'solana-bankrun';

const TOKEN_PROGRAM: typeof TOKEN_2022_PROGRAM_ID | typeof TOKEN_PROGRAM_ID = TOKEN_2022_PROGRAM_ID;
const IDL = require('../target/idl/solearn.json');
const PROGRAM_ID = new PublicKey(IDL.address);

import { Solearn } from "../target/types/solearn";
import { confirmTransaction, createAccountsMintsAndTokenAccounts, makeKeypairs } from '@solana-developers/helpers';

const SECONDS = 1000;

// Tests must complete within half this time otherwise
// they are marked as slow. Since Anchor involves a little
// network IO, these tests usually take about 15 seconds.
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;

const getRandomBigNumber = (size = 8) => {
  return new BN(randomBytes(size));
};


describe("solearn-solana", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Solearn as Program<Solearn>;

  it("Is initialized!", async () => {
    // initialize
    const context = await startAnchor('', [{ name: 'solearn-solana', programId: PROGRAM_ID }], []);
    const provider = new BankrunProvider(context);
    const connection = provider.connection;
    // const payer = provider.wallet as anchor.Wallet;
    const program = new anchor.Program<Solearn>(IDL, provider);

    // We're going to reuse these accounts across multiple tests
    const accounts: Record<string, PublicKey> = {
      tokenProgram: TOKEN_PROGRAM,
    };

    const [alice, bob, tokenMintA, tokenMintB, solearnAccount] = makeKeypairs(5);

    before('Creates Alice and Bob accounts, 2 token mints, and associated token accounts for both tokens for both users', async () => {
      const [aliceTokenAccountA, aliceTokenAccountB, bobTokenAccountA, bobTokenAccountB] = [alice, bob].flatMap((keypair) =>
        [tokenMintA, tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
      );
  
      // Airdrops to users, and creates two tokens mints 'A' and 'B'"
      const minimumLamports = await getMinimumBalanceForRentExemptMint(connection);
  
      const sendSolInstructions: Array<TransactionInstruction> = [alice, bob].map((account) =>
        SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: account.publicKey,
          lamports: 10 * LAMPORTS_PER_SOL,
        }),
      );
  
      const createMintInstructions: Array<TransactionInstruction> = [tokenMintA, tokenMintB].map((mint) =>
        SystemProgram.createAccount({
          fromPubkey: provider.publicKey,
          newAccountPubkey: mint.publicKey,
          lamports: minimumLamports,
          space: MINT_SIZE,
          programId: TOKEN_PROGRAM,
        }),
      );
  
      // Make tokenA and tokenB mints, mint tokens and create ATAs
      const mintTokensInstructions: Array<TransactionInstruction> = [
        {
          mint: tokenMintA.publicKey,
          authority: alice.publicKey,
          ata: aliceTokenAccountA,
        },
        {
          mint: tokenMintB.publicKey,
          authority: bob.publicKey,
          ata: bobTokenAccountB,
        },
      ].flatMap((mintDetails) => [
        createInitializeMint2Instruction(mintDetails.mint, 6, mintDetails.authority, null, TOKEN_PROGRAM),
        createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, mintDetails.ata, mintDetails.authority, mintDetails.mint, TOKEN_PROGRAM),
        createMintToInstruction(mintDetails.mint, mintDetails.ata, mintDetails.authority, 1_000_000_000, [], TOKEN_PROGRAM),
      ]);
  
      // Add all these instructions to our transaction
      const tx = new Transaction();
      tx.instructions = [...sendSolInstructions, ...createMintInstructions, ...mintTokensInstructions];
  
      await provider.sendAndConfirm(tx, [tokenMintA, tokenMintB, alice, bob]);
  
      accounts.admin = alice.publicKey;
      accounts.staking_token =  tokenMintA.publicKey;


    //   #[account(
    //     init, 
    //     // realloc = 8 + wh_account.len(),
    //     // realloc::payer = admin, 
    //     // realloc::zero = false,
    //     payer = admin, 
    //     space = 8 + WorkerHubStorage::LEN,
    //     seeds = [b"worker_hub_storage", sol_learn_account.key().as_ref()], 
    //     bump
    // )]
    // pub sol_learn_account: Account<'info, WorkerHubStorage>,

      // const offer = PublicKey.findProgramAddressSync(
      //   [Buffer.from('vault'), accounts.maker.toBuffer(), offerId.toArrayLike(Buffer, 'le', 8)],
      //   program.programId,
      // )[0];


      // deploy solearn contract
      //     #[account(
      //         init, 
      //         payer = admin, 
      //         space = 8 + VaultAccount::LEN,
      //         seeds = [b"vault", sol_learn_account.key().as_ref()], 
      //         bump
      //     )]
      //     pub vault_wallet_owner: Account<'info, VaultAccount>,
      //     #[account(
      //         init, 
      //         payer = admin, 
      //         space = 8 + Models::LEN,
      //         seeds = [b"models", sol_learn_account.key().as_ref()], 
      //         bump
      //     )]
      //     pub models: Account<'info, Models>,
      //     #[account(
      //         init, 
      //         // realloc = 8 + wh_account.len(),
      //         // realloc::payer = admin, 
      //         // realloc::zero = false,
      //         payer = admin, 
      //         space = 8 + WorkerHubStorage::LEN,
      //         seeds = [b"worker_hub_storage", sol_learn_account.key().as_ref()], 
      //         bump
      //     )]
      //     pub sol_learn_account: Account<'info, WorkerHubStorage>,
      //     pub system_program: Program<'info, System>,
      //     pub sysvar_clock: Sysvar<'info, Clock>,
      // }
      

      // Then determine the account addresses we'll use for the offer and the vault
  
      // 



      // Save the accounts for later use
      accounts.maker = alice.publicKey;
      accounts.taker = bob.publicKey;
      accounts.tokenMintA = tokenMintA.publicKey;
      accounts.makerTokenAccountA = aliceTokenAccountA;
      accounts.takerTokenAccountA = bobTokenAccountA;
      accounts.tokenMintB = tokenMintB.publicKey;
      accounts.makerTokenAccountB = aliceTokenAccountB;
      accounts.takerTokenAccountB = bobTokenAccountB;
    });


    // use token A as EAI

    //   pub fn initialize(
    //     ctx: Context<Initialize>,
    //     min_stake: u64,
    //     reward_per_epoch: u64,
    //     epoch_duration: u64,
    //     miner_minimum_stake: u64,
    //     treasury_address: Pubkey,
    //     fee_l2_percentage: u16,
    //     fee_treasury_percentage: u16,
    //     fee_ratio_miner_validator: u16,
    //     submit_duration: u64,
    //     commit_duration: u64,
    //     reveal_duration: u64,
    //     penalty_duration: u64,
    //     miner_requirement: u8,
    //     blocks_per_epoch: u64,
    //     fine_percentage: u16,
    //     dao_token_reward: u64,
    //     miner_percentage: u16,
    //     user_percentage: u16,
    //     referrer_percentage: u16,
    //     referee_percentage: u16,
    //     l2_owner_percentage: u16,
    // ) -> Result<()> {


    // miner register

    // 

    // unstaking
  });
});

// #[program]
// pub mod escrow {
//     use super::*;

//     pub fn make_offer(
//         context: Context<MakeOffer>,
//         id: u64,
//         token_a_offered_amount: u64,
//         token_b_wanted_amount: u64,
//     ) -> Result<()> {
//         instructions::make_offer::send_offered_tokens_to_vault(&context, token_a_offered_amount)?;
//         instructions::make_offer::save_offer(context, id, token_b_wanted_amount)
//     }

//     pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
//         instructions::take_offer::send_wanted_tokens_to_maker(&context)?;
//         instructions::take_offer::withdraw_and_close_vault(context)
//     }
// }


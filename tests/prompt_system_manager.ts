import { randomBytes } from 'node:crypto';
import { describe, it } from 'node:test';
import * as anchor from '@coral-xyz/anchor';
import { BN, type Program } from '@coral-xyz/anchor';
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from '@solana/spl-token';
import { LAMPORTS_PER_SOL, SYSVAR_CLOCK_PUBKEY, SYSVAR_RENT_PUBKEY, PublicKey, SystemProgram, Transaction, type TransactionInstruction } from '@solana/web3.js';
import { BankrunProvider } from 'anchor-bankrun';
import { assert } from 'chai';
import { startAnchor, Clock } from 'solana-bankrun';

const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
const IDL = require('../target/idl/prompt_system_manager.json');
const PROGRAM_ID = new PublicKey(IDL.address);
const METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

import { PromptSystemManager } from "../target/types/prompt_system_manager";
import { confirmTransaction, createAccountsMintsAndTokenAccounts, makeKeypairs } from '@solana-developers/helpers';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';

const SECONDS = 1000;

// Tests must complete within half this time otherwise
// they are marked as slow. Since Anchor involves a little
// network IO, these tests usually take about 15 seconds.
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;


describe('Prompt System Manager Bankrun test', () => {
  const [admin, alice, bob, tokenMintA, tokenMintB, solearnAccount, model1, model2] = makeKeypairs(8);

  let context, provider, connection, program;
  // We're going to reuse these accounts across multiple tests
  const accounts: Record<string, PublicKey> = {
    tokenProgram: TOKEN_PROGRAM,
    sysvarClock: SYSVAR_CLOCK_PUBKEY,
    systemProgram: SYSTEM_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    rent: SYSVAR_RENT_PUBKEY,
    metadataProgram: METADATA_PROGRAM_ID,
  };

  before('Init Prompt System Manager program', async () => {
    // Then determine the account addresses we'll use for the offer and the vault
  });

  // We'll call this function from multiple tests, so let's seperate it out
  const stake = async () => {
    // fill in here
  };

  const initProgram = async () => {
    context = await startAnchor('', [
      { name: 'prompt_system_manager', programId: PROGRAM_ID },
      // { name: 'token_metadata', programId: METADATA_PROGRAM_ID },
    ], []);
    provider = new BankrunProvider(context);
    connection = provider.connection;

    program = new anchor.Program<PromptSystemManager>(IDL, provider);

    const [aliceTokenAccountA, aliceTokenAccountB, bobTokenAccountA, bobTokenAccountB] = [alice, bob].flatMap((keypair) =>
      [tokenMintA, tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
    );

    // Airdrops to users, and creates two tokens mints 'A' and 'B'"
    const minimumLamports = await getMinimumBalanceForRentExemptMint(connection);

    // 
    const info = await connection.getAccountInfo(METADATA_PROGRAM_ID, "recent");
    console.log({info})

    const sendSolInstructions: Array<TransactionInstruction> = [admin, alice, bob].map((account) =>
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

    // START ADD CODE HERE
    let collection_id = new BN(1);

    accounts.authority = alice.publicKey;
    accounts.payer = alice.publicKey;
    accounts.mint = PublicKey.findProgramAddressSync(
      [Buffer.from('mint'), collection_id.toArrayLike(Buffer, 'le', 8)],
      program.programId,
    )[0];
    accounts.tokenAccount = getAssociatedTokenAddressSync(accounts.mint, accounts.authority, false, TOKEN_PROGRAM);
    accounts.masterEditionAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer(), Buffer.from('edition')],
      accounts.metadataProgram,
    )[0];
    accounts.nftMetadata = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer()],
      accounts.metadataProgram,
    )[0];

    console.log(accounts);

    // mint new collection
    await sendAndConfirmTx(provider, [await program.instruction.createSingleNft(
      collection_id,
      "test",
      "T",
      "test",
      {
        accounts: {...accounts}
      }
    )], [alice]);

    // mint new nft id
    // let nft_id = new BN(1);
    // accounts.collection = accounts.mint;
    // accounts.mint = PublicKey.findProgramAddressSync(
    //   [Buffer.from('mint'), collection_id.toArrayLike(Buffer, 'le', 8), nft_id.toArrayLike(Buffer, 'le', 8)],
    //   program.programId,
    // )[0];
    // accounts.token_account = getAssociatedTokenAddressSync(accounts.mint, accounts.authority, false, TOKEN_PROGRAM);
    // accounts.masterEditionAccount = PublicKey.findProgramAddressSync(
    //   [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer(), Buffer.from('edition')],
    //   accounts.metadataProgram,
    // )[0];
    // accounts.nftMetadata = PublicKey.findProgramAddressSync(
    //   [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer()],
    //   accounts.metadataProgram,
    // )[0];

    // await sendAndConfirmTx(provider, [await program.instruction.mintToCollection(
    //   collection_id,
    //   nft_id,
    //   "T",
    //   "test",
    //   "test",
    //   {
    //     accounts: {...accounts}
    //   }
    // )], [alice]);
    

    // add metadata

    // update metadata
    
  }

  it('Test nft', async () => {
    await initProgram();
    // await stake();
    // Configure the client to use the local cluster.
    
  });

  const sendAndConfirmTx = async (providerAgr: any, insts: any, signers: any) => {
    const initTx = new Transaction();
    initTx.instructions = [...insts];
    await providerAgr.sendAndConfirm(initTx, signers);
  }
});
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
import { startAnchor } from 'solana-bankrun';

const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
const IDL = require('../target/idl/prompt_system_manager.json');
const PROGRAM_ID = new PublicKey(IDL.address);
const METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

import { PromptSystemManager } from "../target/types/prompt_system_manager";
import { makeKeypairs } from '@solana-developers/helpers';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';

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

  const initProgram = async () => {
    context = await startAnchor('', [
      { name: 'prompt_system_manager', programId: PROGRAM_ID },
    ], []);
    provider = anchor.AnchorProvider.env();
    connection = provider.connection;
    anchor.setProvider(provider);
    const payer = provider.wallet as anchor.Wallet;
    program = anchor.workspace.PromptSystemManager as anchor.Program<PromptSystemManager>;

    const [aliceTokenAccountA, aliceTokenAccountB, bobTokenAccountA, bobTokenAccountB] = [alice, bob].flatMap((keypair) =>
      [tokenMintA, tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
    );

    // Airdrops to users, and creates two tokens mints 'A' and 'B'"
    const minimumLamports = await getMinimumBalanceForRentExemptMint(connection);
    const sendSolInstructions: Array<TransactionInstruction> = [admin, alice, bob].map((account) =>
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: account.publicKey,
        lamports: 10 * LAMPORTS_PER_SOL,
      }),
    );

    const createMintInstructions: Array<TransactionInstruction> = [tokenMintA, tokenMintB].map((mint) =>
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
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
      createAssociatedTokenAccountIdempotentInstruction(payer.publicKey, mintDetails.ata, mintDetails.authority, mintDetails.mint, TOKEN_PROGRAM),
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


  //     #[account(
  //         init_if_needed,
  //         payer = payer,
  //         associated_token::mint = mint,
  //         associated_token::authority = payer,
  //     )]
  //     pub token_account: InterfaceAccount<'info, TokenAccount>,
  //     pub associated_token_program: Program<'info, AssociatedToken>,
  //     pub rent: Sysvar<'info, Rent>,
  //     pub system_program: Program<'info, System>,
  //     pub token_program: Program<'info, Token>,
  //     pub metadata_program: Program<'info, Metadata>,
  //     #[account(
  //         mut,
  //         seeds = [
  //             b"metadata".as_ref(),
  //             metadata_program.key().as_ref(),
  //             mint.key().as_ref(),
  //             b"edition".as_ref(),
  //         ],
  //         bump,
  //         seeds::program = metadata_program.key()
  //     )]
  //     /// CHECK:
  //     pub master_edition_account: UncheckedAccount<'info>,
  //     #[account(
  //         mut,
  //         seeds = [
  //             b"metadata".as_ref(),
  //             metadata_program.key().as_ref(),
  //             mint.key().as_ref(),
  //         ],
  //         bump,
  //         seeds::program = metadata_program.key()
  //     )]
  //     /// CHECK:
  //     pub nft_metadata: UncheckedAccount<'info>,
  //     /// CHECK:
  //     pub collection: UncheckedAccount<'info>,
  // }

    let nft_id = new BN(1);
    accounts.collection = accounts.mint;
    
    console.log("fucker");
    console.log(accounts.mint);

    accounts.mint = PublicKey.findProgramAddressSync(
      [Buffer.from('mint'), collection_id.toArrayLike(Buffer, 'le', 8), nft_id.toArrayLike(Buffer, 'le', 8)],
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

    // createAssociatedTokenAccountIdempotentInstruction(alice.publicKey, accounts.token_account, alice.publicKey, accounts.mint, TOKEN_PROGRAM)
    await sendAndConfirmTx(provider, [await program.instruction.mintToCollection(
      collection_id,
      nft_id,
      "test",
      "T",
      "test",
      {
        accounts: {...accounts}
      }
    )], [alice]);
    
    // add metadata
    accounts.promptAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('prompt'), collection_id.toArrayLike(Buffer, 'le', 8), nft_id.toArrayLike(Buffer, 'le', 8), accounts.tokenAccount.toBuffer()],
      program.programId,
    )[0];

    await sendAndConfirmTx(provider, [await program.instruction.addPrompt(
      collection_id,
      nft_id,
      Buffer.from("test add prompt", 'utf8'),
      {
        accounts: {...accounts}
      }
    )], [alice]);


    // update metadata
    await sendAndConfirmTx(provider, [await program.instruction.updatePrompt(
      collection_id,
      nft_id,
      Buffer.from("test update", 'utf8'),
      {
        accounts: {...accounts}
      }
    )], [alice]);
  }

  it('Create an NFT!', async () => {
    await initProgram();
  });

  const sendAndConfirmTx = async (providerAgr: any, insts: any, signers: any) => {
    const initTx = new Transaction();
    initTx.instructions = [...insts];
    await providerAgr.sendAndConfirm(initTx, signers);
  }
});
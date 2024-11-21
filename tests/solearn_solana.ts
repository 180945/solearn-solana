import { randomBytes } from 'node:crypto';
import { describe, it } from 'node:test';
import * as anchor from '@coral-xyz/anchor';
import { BN, type Program } from '@coral-xyz/anchor';
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from '@solana/spl-token';
import { LAMPORTS_PER_SOL, SYSVAR_CLOCK_PUBKEY, PublicKey, SystemProgram, Transaction, type TransactionInstruction } from '@solana/web3.js';
import { BankrunProvider } from 'anchor-bankrun';
import { assert } from 'chai';
import { startAnchor } from 'solana-bankrun';

const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
const IDL = require('../target/idl/solearn.json');
const PROGRAM_ID = new PublicKey(IDL.address);

import { Solearn } from "../target/types/solearn";
import { confirmTransaction, createAccountsMintsAndTokenAccounts, makeKeypairs } from '@solana-developers/helpers';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';

const SECONDS = 1000;

// Tests must complete within half this time otherwise
// they are marked as slow. Since Anchor involves a little
// network IO, these tests usually take about 15 seconds.
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;


describe('Solearn Bankrun example', () => {
  const [admin, alice, bob, tokenMintA, tokenMintB, solearnAccount, model1, model2] = makeKeypairs(8);

  let context, provider, connection, program;
  // We're going to reuse these accounts across multiple tests
  const accounts: Record<string, PublicKey> = {
    tokenProgram: TOKEN_PROGRAM,
    sysvarClock: SYSVAR_CLOCK_PUBKEY,
    systemProgram: SYSTEM_PROGRAM_ID,
  };

  before('Init Solearn program', async () => {
    // Then determine the account addresses we'll use for the offer and the vault
  });

  // We'll call this function from multiple tests, so let's seperate it out
  const stake = async () => {
    // fill in here
  };

  const initProgram = async () => {
    context = await startAnchor('', [{ name: 'solearn_solana', programId: PROGRAM_ID }], []);
    provider = new BankrunProvider(context);
    connection = provider.connection;

    console.log({connection});
    program = new anchor.Program<Solearn>(IDL, provider);

    const [aliceTokenAccountA, aliceTokenAccountB, bobTokenAccountA, bobTokenAccountB] = [alice, bob].flatMap((keypair) =>
      [tokenMintA, tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
    );

    // Airdrops to users, and creates two tokens mints 'A' and 'B'"
    const minimumLamports = await getMinimumBalanceForRentExemptMint(connection);

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

    accounts.admin = admin.publicKey;
    accounts.stakingToken =  tokenMintA.publicKey;
    accounts.solLearnAccount = solearnAccount.publicKey;

    const vault_wallet_owner = PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), solearnAccount.publicKey.toBuffer()],
      program.programId,
    )[0];

    accounts.vaultWalletOwnerPda = vault_wallet_owner;
    accounts.models = PublicKey.findProgramAddressSync(
      [Buffer.from('models'), solearnAccount.publicKey.toBuffer()],
      program.programId,
    )[0];

    const zeroValue = new BN(0);
    await sendAndConfirmTx(provider, [await program.instruction.initialize(
      new BN(1000000),
      new BN(10), // 10 blocks
      new BN(100000000), // EAI decimal 6  => 10 EAI
      bob.publicKey, // treasury address
      new BN(100), // 1% 
      new BN(100), // 1% 
      new BN(5), // dont know this value
      new BN(10), // 10s
      new BN(10), // 10s
      new BN(10), // 10s
      new BN(10), // 10s
      new BN(1),
      new BN(10), // 10 blocks
      new BN(100), // fine 1%
      zeroValue, zeroValue, zeroValue, zeroValue, zeroValue, zeroValue,
      {
        accounts: {...accounts}
      }
    )], [admin, solearnAccount]);

    accounts.minersOfModel = PublicKey.findProgramAddressSync(
      [Buffer.from('models'), solearnAccount.publicKey.toBuffer(), model1.publicKey.toBuffer()],
      program.programId,
    )[0];


    // after init success let add new model 
    await sendAndConfirmTx(provider, [await program.instruction.addModel(
      model1.publicKey,
      {
        accounts: {...accounts}
      }
    )], [admin]);

    // Check our Offer account contains the correct data
    const modelsAccountFetch = await program.account.models.fetch(accounts.models);
    assert(modelsAccountFetch.data.equals(model1.publicKey.toBuffer()))

    accounts.miner = alice.publicKey;
    accounts.minerAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('miner'), accounts.miner.toBuffer(), accounts.solLearnAccount.toBuffer()],
      program.programId,
    )[0];
    accounts.minerStakingWallet = aliceTokenAccountA;
    accounts.vaultStakingWallet = getAssociatedTokenAddressSync(tokenMintA.publicKey, accounts.vaultWalletOwnerPda, true, TOKEN_PROGRAM);

    await sendAndConfirmTx(provider, [createAssociatedTokenAccountIdempotentInstruction(alice.publicKey, accounts.vaultStakingWallet, accounts.vaultWalletOwnerPda, tokenMintA.publicKey, TOKEN_PROGRAM), await program.instruction.minerRegister(
      new BN(100000000),
      {
        accounts: {...accounts}
      }
    )], [alice]);

    // todo: join for minting

    // let clock = await context.getClock();
    // console.log({clock});

    let minerAccount = await program.account.minerAccount.fetch(accounts.minerAccount);
    console.log({minerAccount});


    await sendAndConfirmTx(provider, [await program.instruction.minerUnstake(
      {
        accounts: {...accounts}
      }
    )], [alice]);
    // time travel

    await sendAndConfirmTx(provider, [await program.instruction.minerClaimUnstaked(
      {
        accounts: {...accounts}
      }
    )], [alice]);
  }

  // unstake 
  const unstake = async () => {
    // fill in here
  };


  // claim unstake amount
  const claimUnstakeAmount = async () => {
    // fill in here
  };

  // miner register

  // miner top up

  // miner rejoin

  it('Test staking', async () => {
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
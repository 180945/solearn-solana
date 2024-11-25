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
import { assert, expect } from 'chai';
import { startAnchor, Clock } from 'solana-bankrun';

const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
const IDL = require('../target/idl/solearn.json');
const PROGRAM_ID = new PublicKey(IDL.address);

import { Solearn } from "../target/types/solearn";
import { confirmTransaction, createAccountsMintsAndTokenAccounts, makeKeypairs } from '@solana-developers/helpers';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';
import { beforeEach } from 'mocha';

const SECONDS = 1000;

// Tests must complete within half this time otherwise
// they are marked as slow. Since Anchor involves a little
// network IO, these tests usually take about 15 seconds.
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;

export async function sendAndConfirmTx(providerAgr: any, insts: any, signers: any) {
  const initTx = new Transaction();
  initTx.instructions = [...insts];
  await providerAgr.sendAndConfirm(initTx, signers);
}

// We'll call this function from multiple tests, so let's seperate it out
export async function stake() {
  // fill in here
};

export async function initProgram(_s) {
  _s.context = await startAnchor('', [{ name: 'solearn_solana', programId: PROGRAM_ID }], []);
  _s.provider = new BankrunProvider(_s.context);
  _s.connection = _s.provider.connection;

  console.log({ connection: _s.connection });
  _s.program = new anchor.Program<Solearn>(IDL, _s.provider);

  const [aliceTokenAccountA, aliceTokenAccountB, bobTokenAccountA, bobTokenAccountB] = [_s.alice, _s.bob].flatMap((keypair) =>
    [_s.tokenMintA, _s.tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
  );

  // Airdrops to users, and creates two tokens mints 'A' and 'B'"
  const minimumLamports = await getMinimumBalanceForRentExemptMint(_s.connection);

  const sendSolInstructions: Array<TransactionInstruction> = [_s.admin, _s.alice, _s.bob].map((account) =>
    SystemProgram.transfer({
      fromPubkey: _s.provider.publicKey,
      toPubkey: account.publicKey,
      lamports: 10 * LAMPORTS_PER_SOL,
    }),
  );

  const createMintInstructions: Array<TransactionInstruction> = [_s.tokenMintA, _s.tokenMintB].map((mint) =>
    SystemProgram.createAccount({
      fromPubkey: _s.provider.publicKey,
      newAccountPubkey: mint.publicKey,
      lamports: minimumLamports,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM,
    }),
  );

  // Make tokenA and tokenB mints, mint tokens and create ATAs
  const mintTokensInstructions: Array<TransactionInstruction> = [
    {
      mint: _s.tokenMintA.publicKey,
      authority: _s.alice.publicKey,
      ata: aliceTokenAccountA,
    },
    {
      mint: _s.tokenMintB.publicKey,
      authority: _s.bob.publicKey,
      ata: bobTokenAccountB,
    },
  ].flatMap((mintDetails) => [
    createInitializeMint2Instruction(mintDetails.mint, 6, mintDetails.authority, null, TOKEN_PROGRAM),
    createAssociatedTokenAccountIdempotentInstruction(_s.provider.publicKey, mintDetails.ata, mintDetails.authority, mintDetails.mint, TOKEN_PROGRAM),
    createMintToInstruction(mintDetails.mint, mintDetails.ata, mintDetails.authority, 1_000_000_000, [], TOKEN_PROGRAM),
  ]);

  // Add all these instructions to our transaction
  let tx = new Transaction();
  tx.instructions = [...sendSolInstructions, ...createMintInstructions, ...mintTokensInstructions];

  await _s.provider.sendAndConfirm(tx, [_s.tokenMintA, _s.tokenMintB, _s.alice, _s.bob]);

  _s.accounts.admin = _s.admin.publicKey;
  _s.accounts.stakingToken = _s.tokenMintA.publicKey;
  _s.accounts.solLearnAccount = _s.solearnAccount.publicKey;

  const vault_wallet_owner = PublicKey.findProgramAddressSync(
    [Buffer.from('vault'), _s.solearnAccount.publicKey.toBuffer()],
    _s.program.programId,
  )[0];

  _s.accounts.vaultWalletOwnerPda = vault_wallet_owner;
  _s.accounts.models = PublicKey.findProgramAddressSync(
    [Buffer.from('models'), _s.solearnAccount.publicKey.toBuffer()],
    _s.program.programId,
  )[0];
  _s.accounts.tasks = PublicKey.findProgramAddressSync(
    [Buffer.from('tasks'), _s.solearnAccount.publicKey.toBuffer()],
    _s.program.programId,
  )[0];

  const zeroValue = new BN(0);
  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.initialize(
    new BN(1000000),
    new BN(10), // 10 blocks
    new BN(100000000), // EAI decimal 6  => 10 EAI
    _s.bob.publicKey, // treasury address
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
    new BN(10), // 10s unstaking
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.admin, _s.solearnAccount]);
  _s.accounts.signer = _s.admin.publicKey;
  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.initialize2(
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.admin]);

  _s.accounts.minersOfModel = PublicKey.findProgramAddressSync(
    [Buffer.from('models'), _s.solearnAccount.publicKey.toBuffer(), _s.model1.publicKey.toBuffer()],
    _s.program.programId,
  )[0];


  // after init success let add new model 
  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.addModel(
    _s.model1.publicKey,
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.admin]);

  // Check our Offer account contains the correct data
  const modelsAccountFetch = await _s.program.account.models.fetch(_s.accounts.models);
  assert(modelsAccountFetch.data.equals(_s.model1.publicKey.toBuffer()))

  _s.accounts.miner = _s.alice.publicKey;
  _s.accounts.minerAccount = PublicKey.findProgramAddressSync(
    [Buffer.from('miner'), _s.accounts.miner.toBuffer(), _s.accounts.solLearnAccount.toBuffer()],
    _s.program.programId,
  )[0];
  _s.accounts.minerStakingWallet = aliceTokenAccountA;
  _s.accounts.vaultStakingWallet = getAssociatedTokenAddressSync(_s.tokenMintA.publicKey, _s.accounts.vaultWalletOwnerPda, true, TOKEN_PROGRAM);

  await sendAndConfirmTx(_s.provider, [createAssociatedTokenAccountIdempotentInstruction(_s.alice.publicKey, _s.accounts.vaultStakingWallet, _s.accounts.vaultWalletOwnerPda, _s.tokenMintA.publicKey, TOKEN_PROGRAM), await _s.program.instruction.minerRegister(
    new BN(100000000),
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.alice]);

  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.minerUnstake(
    new BN(0),
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.alice]);

  const currentClock = await _s.context.banksClient.getClock();
  _s.context.setClock(
    new Clock(
      currentClock.slot,
      currentClock.epochStartTimestamp,
      currentClock.epoch,
      currentClock.leaderScheduleEpoch,
      currentClock.unixTimestamp + 11n,
    ),
  );

  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.minerClaimUnstaked(
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.alice]);

  // topup 
  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.topup(
    new BN(100000000),
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.alice]);

  // join minting
  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.joinForMinting(
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.alice]);
    
  // unstake again
  await sendAndConfirmTx(_s.provider, [await _s.program.instruction.minerUnstake(
    new BN(0),
    {
      accounts: { ..._s.accounts }
    }
  )], [_s.alice]);
}

// unstake 
export async function unstake() {
  // fill in here
};


// claim unstake amount
export async function claimUnstakeAmount() {
  // fill in here
};


describe('Solearn Bankrun example', function () {
  const [admin, alice, bob, eve, dom, tokenMintA, tokenMintB, solearnAccount, model1, model2] = makeKeypairs(10);

  let context, provider, connection, program, hmProgram;
  // We're going to reuse these accounts across multiple tests
  const accounts: Record<string, PublicKey> = {
    tokenProgram: TOKEN_PROGRAM,
    sysvarClock: SYSVAR_CLOCK_PUBKEY,
    systemProgram: SYSTEM_PROGRAM_ID,
  };
  let state = {
    admin, alice, bob, eve, dom, tokenMintA, tokenMintB, solearnAccount, model1, model2,
    context, provider, connection, program, hmProgram, accounts
  };
  

  before(async () => {    
    // Then determine the account addresses we'll use for the offer and the vault
  });

  // miner register

  // miner top up

  // miner rejoin

  async function getWorkerHubContract(_s) {
    return _s.program;
  }

  async function getContractInstance(_s, name: string, address: string) {
    return _s.program;
  }

  async function simulateInferAndAssign(_s) {
    const workerHub = _s.program;

    // await sendAndConfirmTx(_s.provider, [await workerHub.instruction.minerRegister(new BN(100000000),
    //   {
    //     accounts: { ..._s.accounts }
    //   })], [_s.eve]);

    // await sendAndConfirmTx(_s.provider, [await workerHub.instruction.joinForMinting({
    //   accounts: { ..._s.accounts }
    // })], [_s.eve]);

    // await sendAndConfirmTx(_s.provider, [await workerHub.instruction.minerRegister(new BN(100000000),
    //   {
    //     accounts: { ..._s.accounts }
    //   })], [_s.dom]);

    // await sendAndConfirmTx(_s.provider, [await workerHub.instruction.joinForMinting({
    //   accounts: { ..._s.accounts }
    // })], [_s.dom]);

    // expect((await workerHub.getMinerAddresses()).length).to.eq(18);
    // simulate contract model call to worker hub to create inference
    // const hybridModelIns = (await getContractInstance(
    //   "HybridModel",
    //   hybridModelAddress
    // )) as HybridModel;
    const creator = _s.alice.publicKey;
    let num = new BN(1);
    _s.accounts.infs = PublicKey.findProgramAddressSync(
      [Buffer.from('inference'), _s.accounts.solLearnAccount.toBuffer(), num.toBuffer('le', 8)],
      _s.program.programId,
    )[0];
    console.log('infs pda', PublicKey.findProgramAddressSync(
      [Buffer.from('inference'), _s.accounts.solLearnAccount.toBuffer(), num.toBuffer('le', 8)],
      _s.program.programId,
    ))
    _s.accounts.referrer = PublicKey.findProgramAddressSync(
      [Buffer.from('referrer'), creator.toBuffer()],
      _s.program.programId,
    )[0];
    _s.accounts.signer = _s.alice.publicKey;

    const modelInput = Buffer.from(randomBytes(32));
    console.log('before infer, model input', modelInput);
    
    await sendAndConfirmTx(_s.provider, [await workerHub.instruction.infer(modelInput, creator,
      new BN(100000), num, _s.model1.publicKey,
      {
        accounts: { ..._s.accounts }
      })], [_s.alice]);
    console.log('done infer');

    // const blockNumber = await getBlockNumber();
    // const block = await getBlock(blockNumber);
    // const blockTime = block?.timestamp || 0;
    // expect inference id to be 1
    expect(await workerHub.instruction.next_inference_id()).to.eq(1);

    const inferInfo = await workerHub.instruction.getInferenceInfo(1n);
    //check inference info
    expect(inferInfo.input).to.eq(modelInput);
    expect(inferInfo.modelAddress).to.eq(hybridModelAddress);

    // expect(inferInfo.submitTimeout).to.eq(blockTime + 600);
    // expect(inferInfo.commitTimeout).to.eq(blockTime + 600 * 2);
    // expect(inferInfo.revealTimeout).to.eq(blockTime + 600 * 3);

    // find the assigned workers
    const assigns = await workerHub.instruction.next_assignment_id();
    const assignedMiners = assigns.map((a) => a.worker);
    expect(assignedMiners.length).to.eq(3);

    return assignedMiners;
  }

  describe('Test staking', async function () {

    
    it('should call infer and get assigned', async function () {
      // setup
      await initProgram(state);
      await simulateInferAndAssign(state);
    });
    // Configure the client to use the local cluster.
    
  });

  
});
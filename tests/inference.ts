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

export async function simulateTx(providerAgr: any, insts: any, signers: any) {
  const initTx = new Transaction();
  initTx.instructions = [...insts];
  return await providerAgr.simulate(initTx, signers);
}

export async function simulateAndGetResponse(providerAgr: any, insts: any, signers: any) {
  const resp = await simulateTx(providerAgr, insts, signers);
  if (resp.returnData.data[1] === 'base64') {
    return Buffer.from(resp.returnData.data[0], 'base64');
  } else {
    console.error(resp.returnData.data[1]);
    throw new Error('Unsupported return data encoding');
  }
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

  const [eveTkn1, eveTkn2, domTkn1, domTkn2] = [_s.eve, _s.dom].flatMap((keypair) =>
    [_s.tokenMintA, _s.tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
  );
  Object.assign(_s, { aliceTokenAccountA, eveTkn1, eveTkn2, domTkn1, domTkn2 });

  // Airdrops to users, and creates two tokens mints 'A' and 'B'"
  const minimumLamports = await getMinimumBalanceForRentExemptMint(_s.connection);

  const sendSolInstructions: Array<TransactionInstruction> = [_s.admin, _s.alice, _s.bob, _s.eve, _s.dom].map((account) =>
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
      ataDetails: [{ ata: aliceTokenAccountA, owner: _s.alice.publicKey }, { ata: eveTkn1, owner: _s.eve.publicKey }, { ata: domTkn1, owner: _s.dom.publicKey }],
    },
    {
      mint: _s.tokenMintB.publicKey,
      authority: _s.bob.publicKey,
      ataDetails: [{ ata: bobTokenAccountB, owner: _s.bob.publicKey }],
    }
  ].flatMap((mintDetails) => [
    createInitializeMint2Instruction(mintDetails.mint, 6, mintDetails.authority, null, TOKEN_PROGRAM),
    ...(mintDetails.ataDetails.map(d => createAssociatedTokenAccountIdempotentInstruction(_s.provider.publicKey, d.ata, d.owner, mintDetails.mint, TOKEN_PROGRAM))),
    ...(mintDetails.ataDetails.map(d => createMintToInstruction(mintDetails.mint, d.ata, mintDetails.authority, 1_000_000_000, [], TOKEN_PROGRAM))),
  ]);

  // Add all these instructions to our transaction
  let tx = new Transaction();
  tx.instructions = [...sendSolInstructions];

  console.log('before sendAndConfirm', [_s.tokenMintA, _s.tokenMintB, _s.alice, _s.bob, _s.eve, _s.dom].map((a) => a.publicKey.toBase58()))
  await _s.provider.sendAndConfirm(tx, []);
  tx = new Transaction();
  console.log('before sendAndConfirm2')
  tx.instructions = [...createMintInstructions, ...mintTokensInstructions];
  await _s.provider.sendAndConfirm(tx, [_s.tokenMintA, _s.tokenMintB, _s.alice, _s.bob]);
  console.log('before sendAndConfirm3')

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
  console.log('before initialize')

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
  console.log('before initialize2')
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

  async function simulateInferAndAssign(_s, _infId: number = 1) {
    const workerHub = _s.program;

    _s.accounts.signer = _s.alice.publicKey;
    console.log('gagaagaggaga', 1);
    // await sendAndConfirmTx(_s.provider, [await workerHub.instruction.minerRegister(
    //   new BN(100000000),
    //   {
    //     accounts: { ..._s.accounts }
    //   }
    // )], [_s.alice]);
    console.log('gagaagaggaga', 2);
    await sendAndConfirmTx(_s.provider, [await workerHub.instruction.joinForMinting(
      {
        accounts: { ..._s.accounts }
      }
    )], [_s.alice]);

    _s.accounts.signer = _s.eve.publicKey;
    _s.accounts.miner = _s.eve.publicKey;
    _s.accounts.minerAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('miner'), _s.accounts.miner.toBuffer(), _s.accounts.solLearnAccount.toBuffer()],
      _s.program.programId,
    )[0];
    _s.accounts.minerStakingWallet = _s.eveTkn1;
    await sendAndConfirmTx(_s.provider, [createAssociatedTokenAccountIdempotentInstruction(_s.eve.publicKey, _s.accounts.vaultStakingWallet, _s.accounts.vaultWalletOwnerPda, _s.tokenMintA.publicKey, TOKEN_PROGRAM), await workerHub.instruction.minerRegister(new BN(100000000),
      {
        accounts: { ..._s.accounts }
      })], [_s.eve]);

    await sendAndConfirmTx(_s.provider, [await workerHub.instruction.joinForMinting({
      accounts: { ..._s.accounts }
    })], [_s.eve]);

    _s.accounts.signer = _s.dom.publicKey;
    _s.accounts.miner = _s.dom.publicKey;
    _s.accounts.minerAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('miner'), _s.accounts.miner.toBuffer(), _s.accounts.solLearnAccount.toBuffer()],
      _s.program.programId,
    )[0];
    _s.accounts.minerStakingWallet = _s.domTkn1;
    await sendAndConfirmTx(_s.provider, [createAssociatedTokenAccountIdempotentInstruction(_s.dom.publicKey, _s.accounts.vaultStakingWallet, _s.accounts.vaultWalletOwnerPda, _s.tokenMintA.publicKey, TOKEN_PROGRAM), await workerHub.instruction.minerRegister(new BN(100000000),
      {
        accounts: { ..._s.accounts }
      })], [_s.dom]);

    await sendAndConfirmTx(_s.provider, [await workerHub.instruction.joinForMinting({
      accounts: { ..._s.accounts }
    })], [_s.dom]);

    // expect((await workerHub.getMinerAddresses()).length).to.eq(18);
    // simulate contract model call to worker hub to create inference
    // const hybridModelIns = (await getContractInstance(
    //   "HybridModel",
    //   hybridModelAddress
    // )) as HybridModel;
    const creator = _s.alice.publicKey;
    _s.accounts.signer = _s.alice.publicKey;
    _s.accounts.miner = _s.alice.publicKey;
    _s.accounts.minerAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('miner'), _s.accounts.miner.toBuffer(), _s.accounts.solLearnAccount.toBuffer()],
      _s.program.programId,
    )[0];
    _s.accounts.minerStakingWallet = _s.aliceTokenAccountA;
    let infId = new BN(_infId);
    
    _s.accounts.infs = PublicKey.findProgramAddressSync(
      [Buffer.from('inference'), infId.toBuffer('le', 8)],
      _s.program.programId,
    )[0];
    _s.accounts.referrer = PublicKey.findProgramAddressSync(
      [Buffer.from('referrer'), creator.toBuffer()],
      _s.program.programId,
    )[0];

    const modelInput = Buffer.from(randomBytes(32));
    
    await sendAndConfirmTx(_s.provider, [await workerHub.instruction.infer(infId, creator,
      modelInput, new BN(100000), _s.model1.publicKey,
      {
        accounts: { ..._s.accounts }
      })], [_s.alice]);

    console.log('infer call finished')
    // TODO: assert token balances
    // const blockNumber = await getBlockNumber();
    // const block = await getBlock(blockNumber);
    // const blockTime = block?.timestamp || 0;
    // expect inference id to be 1
    
    const nextInfId = new BN(
      await simulateAndGetResponse(_s.provider,
        [await workerHub.instruction.nextInferenceId({ accounts: { ..._s.accounts } })],
        [_s.provider.wallet.payer]
      ),
      undefined, 'le',
    );
    // console.log('next_inf_id', nextInfId);
    expect(nextInfId.toNumber()).to.eq(2);

    let taskCount = new BN(
      await simulateAndGetResponse(_s.provider,
        [await workerHub.instruction.getTaskCount({ accounts: { ..._s.accounts } })],
        [_s.provider.wallet.payer]
      ),
      undefined, 'le',
    );
    console.log('after calling infer, taskCount =', taskCount.toNumber());
    expect(taskCount.toNumber()).to.eq(1);

    let assignmentId = new BN(1);
    _s.accounts.assignment = PublicKey.findProgramAddressSync(
      [Buffer.from('assignment'), assignmentId.toBuffer('le', 8)],
      _s.program.programId,
    )[0];
    await sendAndConfirmTx(_s.provider, [await workerHub.instruction.createAssignment(assignmentId,
      {
        accounts: { ..._s.accounts }
      })], [_s.alice]);

    taskCount = new BN(
      await simulateAndGetResponse(_s.provider,
        [await workerHub.instruction.getTaskCount({ accounts: { ..._s.accounts } })],
        [_s.provider.wallet.payer]
      ),
      undefined, 'le',
    );
    console.log('after calling createAssignment, taskCount =', taskCount.toNumber());
    expect(taskCount.toNumber()).to.eq(0);

    const buf = await simulateAndGetResponse(_s.provider,
      [await workerHub.instruction.getAssignment(assignmentId, "worker", { accounts: { ..._s.accounts } })],
      [_s.provider.wallet.payer]
    );
    let len = buf.readUInt32LE(0);
    if (len > 4 + buf.length) {
      throw new Error('Invalid getAssignment data length');
    }
    const readData = buf.subarray(4, 4 + len);
    let assignedWorkerPubkey = new PublicKey(
      readData
    );
    // console.log('assignedWorkerPubkey', assignedWorkerPubkey.toBase58());
    // console.log('vs potential miners', _s.alice.publicKey.toBase58(), _s.eve.publicKey.toBase58(), _s.dom.publicKey.toBase58());

    // const inferInfo = await workerHub.instruction.getInferenceInfo(1n);
    //check inference info
    // expect(inferInfo.input).to.eq(modelInput);
    // expect(inferInfo.modelAddress).to.eq(hybridModelAddress);

    // expect(inferInfo.submitTimeout).to.eq(blockTime + 600);
    // expect(inferInfo.commitTimeout).to.eq(blockTime + 600 * 2);
    // expect(inferInfo.revealTimeout).to.eq(blockTime + 600 * 3);

    const nextAssignmentId = new BN(
      await simulateAndGetResponse(_s.provider,
        [await workerHub.instruction.nextAssignmentId({ accounts: { ..._s.accounts } })],
        [_s.provider.wallet.payer]
      ),
      undefined, 'le',
    );
    // console.log('next_a_id', nextAssignmentId);
    expect(nextAssignmentId.toNumber()).to.eq(2);

    const assignedMiners = [_s.alice, _s.eve, _s.dom].filter((miner) => miner.publicKey.toBase58() === assignedWorkerPubkey.toBase58());
    expect(assignedMiners.length).to.eq(1);

    return assignedMiners;
  }

  describe('Test staking', async function () {
    let assignedMiner;
    let inferenceId = 1;
    let assignmentId = 1;
    
    it('should call infer and get assigned', async function () {
      await initProgram(state);
      const res = await simulateInferAndAssign(state, inferenceId);
      console.log('infer & assigned', res.map(m => m.publicKey.toBase58()));
      assignedMiner = res[0];
    });

    it('should topup inference', async function () {
      console.log('begin topup inference');
      state.accounts.signer = state.alice.publicKey;
      state.accounts.miner = state.alice.publicKey;
      state.accounts.minerAccount = PublicKey.findProgramAddressSync(
        [Buffer.from('miner'), state.accounts.miner.toBuffer(), state.accounts.solLearnAccount.toBuffer()],
        state.program.programId,
      )[0];
      await sendAndConfirmTx(state.provider, [await state.program.instruction.topupInfer(
        new BN(1),
        new BN(100000000),
        {
          accounts: { ...state.accounts }
        }
      )], [state.alice]);
    });
    it("should seize the miner role", async () => {
      state.accounts.signer = assignedMiner.publicKey;
      state.accounts.miner = assignedMiner.publicKey;
      state.accounts.minerAccount = PublicKey.findProgramAddressSync(
        [Buffer.from('miner'), state.accounts.miner.toBuffer(), state.accounts.solLearnAccount.toBuffer()],
        state.program.programId,
      )[0];
      state.accounts.daoReceiverInfos = PublicKey.findProgramAddressSync(
        [Buffer.from('dao_receiver_infos'), state.accounts.solLearnAccount.toBuffer()],
        state.program.programId,
      )[0]; 
      state.accounts.votingInfo = PublicKey.findProgramAddressSync(
        [Buffer.from('voting_info'), new BN(inferenceId).toBuffer('le', 8)],
        state.program.programId,
      )[0];         
      await sendAndConfirmTx(state.provider, [await state.program.instruction.seizeMinerRole(
        new BN(assignmentId),
        new BN(inferenceId),
        {
          accounts: { ...state.accounts }
        }
      )], [assignedMiner]);

      // this should fail
      // expect(await sendAndConfirmTx(state.provider, [await state.program.instruction.seizeMinerRole(
      //   new BN(1),
      //   new BN(1),
      //   {
      //     accounts: { ...state.accounts }
      //   }
      // )], [assignedMiner])).to.throw('inference has been seized');
    });

    it("should submit solution", async () => {
      state.accounts.signer = assignedMiner.publicKey;
      state.accounts.miner = assignedMiner.publicKey;
      state.accounts.minerAccount = PublicKey.findProgramAddressSync(
        [Buffer.from('miner'), state.accounts.miner.toBuffer(), state.accounts.solLearnAccount.toBuffer()],
        state.program.programId,
      )[0];
      state.accounts.daoReceiverInfos = PublicKey.findProgramAddressSync(
        [Buffer.from('dao_receiver_infos'), state.accounts.solLearnAccount.toBuffer(), new BN(inferenceId).toBuffer('le', 8)],
        state.program.programId,
      )[0]; 
      state.accounts.votingInfo = PublicKey.findProgramAddressSync(
        [Buffer.from('voting_info'), new BN(inferenceId).toBuffer('le', 8)],
        state.program.programId,
      )[0];         
      const solution = Buffer.from("solution for test");
      await sendAndConfirmTx(state.provider, [await state.program.instruction.submitSolution(new BN(inferenceId), new BN(assignmentId), solution, {
        accounts: { ...state.accounts }
      })], [assignedMiner]);
    });
      
    
  });

  
});
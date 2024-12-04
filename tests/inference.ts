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
import { keccak_256 } from "js-sha3";

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

export async function makeBlock(provider: any) {
  const tx = new Transaction();
  tx.add(SystemProgram.transfer({
    fromPubkey: provider.publicKey,
    toPubkey: provider.publicKey,
    lamports: 1,
  }));
  const currentClock = await provider.context.banksClient.getClock();
  provider.context.setClock(
    new Clock(
      currentClock.slot + 1n,
      currentClock.epochStartTimestamp,
      currentClock.epoch,
      currentClock.leaderScheduleEpoch,
      currentClock.unixTimestamp + 11n,
    ),
  );

  await provider.sendAndConfirm(tx, [provider.wallet.payer]);
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

  const [aliceTokenAccountA, aliceTokenAccountB, bobTokenAccountA, bobTokenAccountB, adminTokenAccountA, _] = [_s.alice, _s.bob, _s.admin].flatMap((keypair) =>
    [_s.tokenMintA, _s.tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
  );

  const [eveTkn1, eveTkn2, domTkn1, domTkn2] = [_s.eve, _s.dom].flatMap((keypair) =>
    [_s.tokenMintA, _s.tokenMintB].map((mint) => getAssociatedTokenAddressSync(mint.publicKey, keypair.publicKey, false, TOKEN_PROGRAM)),
  );
  

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
      ataDetails: [{ ata: aliceTokenAccountA, owner: _s.alice.publicKey }, { ata: eveTkn1, owner: _s.eve.publicKey }, { ata: domTkn1, owner: _s.dom.publicKey }, { ata: bobTokenAccountA, owner: _s.bob.publicKey }, { ata: adminTokenAccountA, owner: _s.admin.publicKey }],
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
  let tokenAccountToOwner = {}
  tokenAccountToOwner[aliceTokenAccountA.toBase58()] = _s.alice;
  tokenAccountToOwner[eveTkn1.toBase58()] = _s.eve;
  tokenAccountToOwner[domTkn1.toBase58()] = _s.dom;
  tokenAccountToOwner[bobTokenAccountA.toBase58()] = _s.bob;
  tokenAccountToOwner[adminTokenAccountA.toBase58()] = _s.admin;
  let ownerToTokenAccount = {}
  ownerToTokenAccount[_s.alice.publicKey.toBase58()] = aliceTokenAccountA;
  ownerToTokenAccount[_s.eve.publicKey.toBase58()] = eveTkn1;
  ownerToTokenAccount[_s.dom.publicKey.toBase58()] = domTkn1;
  ownerToTokenAccount[_s.bob.publicKey.toBase58()] = bobTokenAccountA;
  ownerToTokenAccount[_s.admin.publicKey.toBase58()] = adminTokenAccountA;
  Object.assign(_s, { aliceTokenAccountA, eveTkn1, eveTkn2, domTkn1, domTkn2, adminTokenAccountA, bobTokenAccountA, tokenAccountToOwner, ownerToTokenAccount });

  // Add all these instructions to our transaction
  let tx = new Transaction();
  tx.instructions = [...sendSolInstructions, ...createMintInstructions];

  await _s.provider.sendAndConfirm(tx, [_s.tokenMintA, _s.tokenMintB]);
  tx = new Transaction();
  tx.instructions = [...mintTokensInstructions];
  await _s.provider.sendAndConfirm(tx, [_s.alice, _s.bob]);

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
    new BN(3),
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
    context, provider, connection, program, hmProgram, accounts, aliceTokenAccountA: undefined
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
    // await sendAndConfirmTx(_s.provider, [await workerHub.instruction.minerRegister(
    //   new BN(100000000),
    //   {
    //     accounts: { ..._s.accounts }
    //   }
    // )], [_s.alice]);
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
    // const creator = _s.aliceTokenAccountA.publicKey;
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
      [Buffer.from('referrer'), _s.alice.publicKey.toBuffer()],
      _s.program.programId,
    )[0];
    state.accounts.daoReceiverInfos = PublicKey.findProgramAddressSync(
      [Buffer.from('dao_receiver_infos'), state.accounts.solLearnAccount.toBuffer(), new BN(infId).toBuffer('le', 8)],
      state.program.programId,
    )[0]; 
    state.accounts.votingInfo = PublicKey.findProgramAddressSync(
      [Buffer.from('voting_info'), new BN(infId).toBuffer('le', 8)],
      state.program.programId,
    )[0];

    const modelInput = Buffer.from(randomBytes(32));
    
    await sendAndConfirmTx(_s.provider, [await workerHub.instruction.infer(infId, _s.aliceTokenAccountA,
      modelInput, new BN(100000), _s.model1.publicKey,
      {
        accounts: { ..._s.accounts }
      })], [_s.alice]);

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
    expect(taskCount.toNumber()).to.eq(3);

    
    for (let i = 1; i <= 3; i++) {
      let assignmentId = new BN(i);
      _s.accounts.assignment = PublicKey.findProgramAddressSync(
        [Buffer.from('assignment'), assignmentId.toBuffer('le', 8)],
        _s.program.programId,
      )[0];
      await sendAndConfirmTx(_s.provider, [await workerHub.instruction.createAssignment(assignmentId,
        {
          accounts: { ..._s.accounts }
        })], [_s.alice]);
    }

    taskCount = new BN(
      await simulateAndGetResponse(_s.provider,
        [await workerHub.instruction.getTaskCount({ accounts: { ..._s.accounts } })],
        [_s.provider.wallet.payer]
      ),
      undefined, 'le',
    );
    console.log('after calling createAssignment, taskCount =', taskCount.toNumber());
    expect(taskCount.toNumber()).to.eq(0);

    let assignedPubkey = [];
    for (let i = 1; i <= 3; i++) {
      let assignmentId = new BN(i);
      _s.accounts.assignment = PublicKey.findProgramAddressSync(
        [Buffer.from('assignment'), assignmentId.toBuffer('le', 8)],
        _s.program.programId,
      )[0];
      const buf = await simulateAndGetResponse(_s.provider,
        [await workerHub.instruction.getAssignment(new BN(i), "worker", { accounts: { ..._s.accounts } })],
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
      assignedPubkey.push(assignedWorkerPubkey);
    }
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
    expect(nextAssignmentId.toNumber()).to.eq(4);

    const minerLst = [_s.alice, _s.eve, _s.dom];
    let pubkeyToMiner = {};
    for (let i = 0; i < minerLst.length; i++) {
      pubkeyToMiner[minerLst[i].publicKey.toBase58()] = minerLst[i];
    }

    const assignedMiners = assignedPubkey.map((pubkey) => pubkeyToMiner[pubkey.toBase58()]);
    expect(assignedMiners.length).to.eq(3);

    return assignedMiners;
  }

  describe('Test staking', async function () {
    let assignedMiner;
    let valMiners;
    let inferenceId = 1;
    let miningAssignmentId = 1;
    let valAssignmentIds = [2, 3];
    const nonces = [42, 4242];
    let solution = Buffer.from("solution for test");
    
    it('should call infer and get assigned', async function () {
      await initProgram(state);
      const res = await simulateInferAndAssign(state, inferenceId);
      // console.log('infer & assigned', res.map(m => m.publicKey.toBase58()));
      let accountExists = {};
      for (let k of Object.values(state)) {
        if (k?.publicKey?.toBase58()) accountExists[k?.publicKey?.toBase58()] = true;
      }
      let dup = {};
      for (let i = 0; i < res.length; i++) {
        // console.log('look for', res[i].publicKey.toBase58());
        // console.log('in accountExists', accountExists);
        expect(accountExists[res[i].publicKey.toBase58()]).to.be.true;
        expect(dup[res[i].publicKey.toBase58()]).to.be.undefined;
        dup[res[i].publicKey.toBase58()] = true;
      }
      assignedMiner = res[0];
      valMiners = res.slice(1);
      console.log('assignedMiner', assignedMiner.publicKey.toBase58());
      console.log('valMiners', valMiners.map(m => m.publicKey.toBase58()));
    });

    it('should topup inference', async function () {
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
      state.accounts.assignment = PublicKey.findProgramAddressSync(
        [Buffer.from('assignment'), new BN(miningAssignmentId).toBuffer('le', 8)],
        state.program.programId,
      )[0];

      await sendAndConfirmTx(state.provider, [await state.program.instruction.seizeMinerRole(
        new BN(miningAssignmentId),
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
      state.accounts.assignment = PublicKey.findProgramAddressSync(
        [Buffer.from('assignment'), new BN(miningAssignmentId).toBuffer('le', 8)],
        state.program.programId,
      )[0];
      
      await sendAndConfirmTx(state.provider, [await state.program.instruction.submitSolution(new BN(miningAssignmentId), new BN(inferenceId), solution, {
        accounts: { ...state.accounts }
      })], [assignedMiner]);
    });

    it('should commit', async () => {
      for (let i = 0; i < nonces.length; i++) {
        
        let buf = Buffer.concat([new BN(nonces[i]).toBuffer('le', 8), valMiners[i].publicKey.toBuffer(), solution]);
        // console.log('commit content', buf);
        const commitment = Buffer.from(keccak_256.update(buf).digest());

        // const commitment = ethers.solidityPackedKeccak256(['uint64', 'bytes32', 'bytes'], [nonces[i], valMiners[i].publicKey.toBuffer(), ethers.hexlify(solution)]);
        state.accounts.signer = valMiners[i].publicKey;
        state.accounts.miner = valMiners[i].publicKey;
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
        state.accounts.assignment = PublicKey.findProgramAddressSync(
          [Buffer.from('assignment'), new BN(valAssignmentIds[i]).toBuffer('le', 8)],
          state.program.programId,
        )[0];

        for (let i = 0; i < 2; i++) {
          await makeBlock(state.provider);
          // sleep for 200ms
          await new Promise(r => setTimeout(r, 200));
        }
        await sendAndConfirmTx(state.provider, [await state.program.instruction.commit(new BN(valAssignmentIds[i]), new BN(inferenceId), commitment, {
          accounts: { ...state.accounts }
        })], [valMiners[i]]);
      }
    });

    it('should reveal', async () => {
      for (let i = 0; i < nonces.length; i++) {
        state.accounts.signer = valMiners[i].publicKey;
        state.accounts.miner = valMiners[i].publicKey;
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
        state.accounts.assignment = PublicKey.findProgramAddressSync(
          [Buffer.from('assignment'), new BN(valAssignmentIds[i]).toBuffer('le', 8)],
          state.program.programId,
        )[0];
        state.accounts.tokenRecipient = state.aliceTokenAccountA;

        for (let i = 0; i < 10; i++) {
          await makeBlock(state.provider);
          // sleep for 200ms
          await new Promise(r => setTimeout(r, 200));
        }
        await sendAndConfirmTx(state.provider, [await state.program.instruction.reveal(new BN(valAssignmentIds[i]), new BN(inferenceId), new BN(nonces[i]), solution, {
          accounts: { ...state.accounts }
        })], [valMiners[i]]);
      }
    });
      
    it("should resolve inference", async () => {
      state.accounts.signer = state.alice.publicKey;
      state.accounts.miner = state.alice.publicKey;
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
      state.accounts.assignment = PublicKey.findProgramAddressSync(
        [Buffer.from('assignment'), new BN(miningAssignmentId).toBuffer('le', 8)],
        state.program.programId,
      )[0];
      state.accounts.tokenRecipient = state.aliceTokenAccountA;
      // console.log('state.accounts.solLearnAccount', state.accounts.solLearnAccount.toBase58(), state.accounts.tokenRecipient);
      for (let i = 0; i < 10; i++) {
        await makeBlock(state.provider);
        // sleep for 200ms
        await new Promise(r => setTimeout(r, 200));
      }
      
      await sendAndConfirmTx(state.provider, [await state.program.instruction.resolveInference(new BN(miningAssignmentId), new BN(inferenceId), {
        accounts: { ...state.accounts }
      })], [state.alice]);
      let taskCount = new BN(
        await simulateAndGetResponse(state.provider,
          [await state.program.instruction.getTaskCount({ accounts: { ...state.accounts } })],
          [state.provider.wallet.payer]
        ),
        undefined, 'le',
      );
      expect(taskCount.toNumber()).to.eq(5); // 3 miners, l2_owner, treasury
    });

    it('should pay miners after inference resolution', async function () {
      const recipients = [assignedMiner, valMiners[0], valMiners[1], state.admin, state.bob];
      const potentialMiners = [state.alice, state.eve, state.dom];
      // console.log('recipients', recipients.map(r => r.publicKey.toBase58()), 'potential miners', potentialMiners.map(p => p.publicKey.toBase58()));
      const tokenAccounts = [state.aliceTokenAccountA, state.eveTkn1, state.domTkn1, state.adminTokenAccountA, state.bobTokenAccountA];
      // console.log('tokenAccounts', tokenAccounts.map(t => t.toBase58()));
      const workerPubkey = [undefined, undefined, undefined, state.admin.publicKey, state.bob.publicKey];
      // console.log('recipients', state.ownerToTokenAccount);
      for (let i = 0; i < 5; i++) {
        let assignmentId = new BN(1)
        console.log('paying miner', i);
        let assignedWorkerPubkey;
        if (workerPubkey[i]) {
          assignedWorkerPubkey = workerPubkey[i];
          state.accounts.assignment = PublicKey.findProgramAddressSync(
            [Buffer.from('assignment'), assignmentId.toBuffer('le', 8)],
            state.program.programId,
          )[0];
        } else {
          assignmentId = new BN(i + 1);
          state.accounts.assignment = PublicKey.findProgramAddressSync(
            [Buffer.from('assignment'), assignmentId.toBuffer('le', 8)],
            state.program.programId,
          )[0];
          const buf = await simulateAndGetResponse(state.provider,
            [await state.program.instruction.getAssignment(assignmentId, "worker", { accounts: { ...state.accounts } })],
            [state.provider.wallet.payer]
          );
          let len = buf.readUInt32LE(0);
          if (len > 4 + buf.length) {
            throw new Error('Invalid getAssignment data length');
          }
          const readData = buf.subarray(4, 4 + len);
          assignedWorkerPubkey = new PublicKey(
            readData
          );

        }
        
        state.accounts.tokenRecipient = state.ownerToTokenAccount[assignedWorkerPubkey];
        // console.log('tokenRecipient', state.accounts.tokenRecipient.toBase58());
        await sendAndConfirmTx(state.provider, [await state.program.instruction.payMiner(assignmentId,
          {
            accounts: { ...state.accounts }
          })], []);
      }
      let taskCount = new BN(
        await simulateAndGetResponse(state.provider,
          [await state.program.instruction.getTaskCount({ accounts: { ...state.accounts } })],
          [state.provider.wallet.payer]
        ),
        undefined, 'le',
      );
      expect(taskCount.toNumber()).to.eq(0);
    });
  });

  
});
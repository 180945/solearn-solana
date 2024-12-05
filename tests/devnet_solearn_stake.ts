import { describe, it } from 'node:test';
import * as anchor from '@coral-xyz/anchor';
import { BN } from '@coral-xyz/anchor';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  getAssociatedTokenAddressSync,
} from '@solana/spl-token';
import {Keypair, SYSVAR_CLOCK_PUBKEY, SYSVAR_RENT_PUBKEY, PublicKey, sendAndConfirmTransaction, Transaction} from '@solana/web3.js';
const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
const IDL = require('../target/idl/solearn.json');
IDL.address = new PublicKey(process.env.SOLEARN_PROGRAM_ID ? process.env.SOLEARN_PROGRAM_ID : IDL.address );
import { Solearn } from "../target/types/solearn";
import bs58 from 'bs58';
const METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';

describe('Solearn Staking', () => {
  let program;
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
    anchor.setProvider(anchor.AnchorProvider.env());
    // program = anchor.workspace.Solearn as Program<Solearn>;
    program = new anchor.Program<Solearn>(IDL, anchor.AnchorProvider.env());
    const programProvider = program.provider as anchor.AnchorProvider;
    // const minimumLamports = await getMinimumBalanceForRentExemptMint(connection);
    const adminImportded = programProvider.wallet;
    const admin = Keypair.fromSecretKey((adminImportded as NodeWallet).payer.secretKey);

    const connection = programProvider.connection;

    const stakingTokenPubKey = new PublicKey("9XiufKRNgX2ZKtTxY5eejVVvTsXLsr2q9VAr4iwGBAju");
    const solLearnAccount = new PublicKey(process.env.SOLEARN_ACCOUNT);

    console.log("solearn account: ", solLearnAccount);

    accounts.admin = admin.publicKey;
    accounts.stakingToken = stakingTokenPubKey;
    accounts.solLearnAccount = solLearnAccount;

    const vault_wallet_owner = PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), accounts.solLearnAccount.toBuffer()],
      program.programId,
    )[0];

    accounts.vaultWalletOwnerPda = vault_wallet_owner;
    accounts.models = PublicKey.findProgramAddressSync(
      [Buffer.from('models'), accounts.solLearnAccount.toBuffer()],
      program.programId,
    )[0];
    accounts.vaultStakingWallet = getAssociatedTokenAddressSync(accounts.stakingToken, accounts.vaultWalletOwnerPda, true, TOKEN_PROGRAM);

    // add model
    const modelPubkey = new PublicKey("2hi9QXnNRsgFit3SEn9AHUzENn1SrCt7KoHG9QK63ynA");
    accounts.minersOfModel = PublicKey.findProgramAddressSync(
      [Buffer.from('models'), accounts.solLearnAccount.toBuffer(), modelPubkey.toBuffer()],
      program.programId,
    )[0];
    const addModelInst = await program.instruction
      .addModel(
        modelPubkey,
        {
          accounts: {...accounts}
        }
      );


    // create vault staking wallet 
    const transaction = new Transaction().add(
      addModelInst,
      createAssociatedTokenAccountIdempotentInstruction(admin.publicKey, accounts.vaultStakingWallet, accounts.vaultWalletOwnerPda, accounts.stakingToken, TOKEN_PROGRAM),
    );

    // Sign transaction, broadcast, and confirm
    const txAddModel =  await sendAndConfirmTransaction(connection, transaction, [admin]);
    console.log({txAddModel});

    // stake and join minting
    const minerList = [process.env.MINER_1, process.env.MINER_2, process.env.MINER_3];
    for (let i = 0; i < minerList.length; i++) {
      await StakeAndJoinMinting(connection, accounts, program, minerList[i], modelPubkey);
    }
  }

  // temporary hardcode => query it after register success
  const StakeAndJoinMinting = async (connection: any, accounts: any, program: any, miner: any, modelOfMinerWhenRegistered: any) => {
    const minerKeypair = Keypair.fromSecretKey(bs58.decode(miner));
    accounts.miner = minerKeypair.publicKey;
    accounts.minerAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('miner'), accounts.miner.toBuffer(), accounts.solLearnAccount.toBuffer()],
      program.programId,
    )[0];
    accounts.minerStakingWallet = getAssociatedTokenAddressSync(accounts.stakingToken, accounts.miner, false, TOKEN_PROGRAM);
    const registerInst = await program.instruction
      .minerRegister(
        new BN(1000000),
        {
          accounts: {...accounts}
        }
      );

    const joinForMintingInst = await program.instruction
      .joinForMinting(
        {
          accounts: {...accounts}
        }
      );

    // create vault staking wallet 
    const transaction = new Transaction().add(
      registerInst, joinForMintingInst,
    );

    // Sign transaction, broadcast, and confirm
    await sendAndConfirmTransaction(connection, transaction, [minerKeypair]);
  }


  it('Devnet Create an NFT!', async () => {
    await initProgram();
  });
});
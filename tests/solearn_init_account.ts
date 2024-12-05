
import { describe, it } from 'node:test';
import * as anchor from '@coral-xyz/anchor';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet'
import { BN, type Program } from '@coral-xyz/anchor';
import {
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { SYSVAR_CLOCK_PUBKEY, PublicKey, Keypair} from '@solana/web3.js';

const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
const IDL = require('../target/idl/solearn.json');
// IDL.address = new PublicKey("8fXqHtGHRwr7Mdif7HJhsV66qwSWLnozjabo2uEHPFZ1");
import { Solearn } from "../target/types/solearn";
import { makeKeypairs } from '@solana-developers/helpers';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';

const SECONDS = 1000;

// Tests must complete within half this time otherwise
// they are marked as slow. Since Anchor involves a little
// network IO, these tests usually take about 15 seconds.
const ANCHOR_SLOW_TEST_THRESHOLD = 40 * SECONDS;

describe('Solearn Deploy', () => {
  const [bob, solearnAccount, model1, model2] = makeKeypairs(8);

  let provider, connection, program;
  // We're going to reuse these accounts across multiple tests
  const accounts: Record<string, PublicKey> = {
    tokenProgram: TOKEN_PROGRAM,
    sysvarClock: SYSVAR_CLOCK_PUBKEY,
    systemProgram: SYSTEM_PROGRAM_ID,
  };

  before('Init Solearn program', async () => {
    // Then determine the account addresses we'll use for the offer and the vault
  });

  const initProgram = async () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    // program = anchor.workspace.Solearn as Program<Solearn>;
    program = new anchor.Program<Solearn>(IDL, anchor.AnchorProvider.env());
    const programProvider = program.provider as anchor.AnchorProvider;
    // const minimumLamports = await getMinimumBalanceForRentExemptMint(connection);
    const adminImportded = programProvider.wallet;
    const admin = Keypair.fromSecretKey((adminImportded as NodeWallet).payer.secretKey);

    const stakingTokenPubKey = new PublicKey("9XiufKRNgX2ZKtTxY5eejVVvTsXLsr2q9VAr4iwGBAju");

    accounts.admin = admin.publicKey;
    accounts.stakingToken = stakingTokenPubKey;
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
    const tx = await program.methods
        .initialize(
            new BN(1000000),
            new BN(10000), // 10000s
            new BN(1000000), // EAI decimal 6  => 1 EAI
            bob.publicKey, // treasury address
            new BN(100), // 1% 
            new BN(100), // 1% 
            new BN(5), // dont know this value
            new BN(1000), // 1000s
            new BN(1000), // 1000s
            new BN(1000), // 1000s
            new BN(1000), // 1000s
            new BN(3),
            new BN(100), // fine 1%
            zeroValue, zeroValue, zeroValue, zeroValue, zeroValue, zeroValue, 
            new BN(120), // 120s unstaking
        )
        .accounts({
            ...accounts
        })
        .signers([admin, solearnAccount])
        .rpc();

    console.log({tx});

    console.log("treary: ", bob);
    console.log("solearn account: ", solearnAccount);
  }

  it('Devnet Test init account on testnet', async () => {
    await initProgram();    
  });
});
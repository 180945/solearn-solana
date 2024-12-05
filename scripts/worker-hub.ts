import { AnchorProvider, Program, Wallet, setProvider } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection, PublicKey, Keypair, SystemProgram, SYSVAR_CLOCK_PUBKEY, TransactionInstruction, Transaction, LAMPORTS_PER_SOL } from "@solana/web3.js";
import type { Solearn } from "../target/types/solearn";
// import type { BasicToken } from "../target/types/basic_token";
import solearnIdl from "../target/idl/solearn.json";
// import tokenIdl from "../target/idl/basic_token.json";
import { BN } from "bn.js";
import { makeKeypairs } from '@solana-developers/helpers';
import {
    createAssociatedTokenAccountIdempotentInstruction,
    createInitializeMint2Instruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync,
    getMinimumBalanceForRentExemptMint,
    MINT_SIZE,
    TOKEN_PROGRAM_ID,
} from '@solana/spl-token';

const PRIVKEY_BYTES = [];

const providerAccountKeypair = Keypair.fromSecretKey(new Uint8Array(PRIVKEY_BYTES));
// console.log("newAccountKp", newAccountKp);
const url = clusterApiUrl("devnet") // "https://api.devnet.solana.com"
const connection = new Connection(url, "confirmed");
const wallet = new Wallet(providerAccountKeypair);
console.log("wallet", wallet.publicKey.toBase58());

const provider = new AnchorProvider(connection, wallet, {});
setProvider(provider);
 
export const solearnProgram = new Program(solearnIdl as Solearn, provider);
// export const tokenProgram = new Program(tokenIdl as BasicToken, provider);
console.log("Program", solearnProgram.idl.address)
// console.log("Token Program", tokenProgram.programId);
let id = new BN(1);
// const [treasury, solLearnAccount, model1] = makeKeypairs(3);
// console.log('treasury', treasury);
// console.log('solLearnAccount', solLearnAccount);
// console.log('model1', model1);
const treasury = Keypair.fromSecretKey(new Uint8Array([]));
const solLearnAccount = Keypair.fromSecretKey(new Uint8Array([]));
// Keypair.fromSecretKey(new Uint8Array([13, 174, 61, 148, 172, 115, 95, 215, 238, 210, 229, 220, 20, 31, 232, 16, 41, 144, 185, 18, 111, 25, 30, 184, 99, 114, 216, 112, 211, 215, 88, 71, 58, 216, 22, 117, 128, 176, 219, 107, 67, 192, 216, 159, 250, 28, 229, 156, 112, 145, 96, 196, 94, 227, 249, 162, 187, 86, 44, 31, 188, 99, 208, 171]))
const model1 = Keypair.fromSecretKey(new Uint8Array([]))
console.log('model1 pubkey', model1.publicKey.toBase58());
// const [acc1, token1] = makeKeypairs(2);
// console.log('acc1', acc1);
// console.log('token1', token1, 'pubkey', token1.publicKey.toBase58());
const acc1 = Keypair.fromSecretKey(new Uint8Array([]));
const token1 = Keypair.fromSecretKey(new Uint8Array([]));
// const [tokenAccount1] = [acc1].map((keypair) =>
//     getAssociatedTokenAddressSync(token1.publicKey, keypair.publicKey, false, TOKEN_PROGRAM_ID),
// );
const [tokenAccount0, tokenAccount1] = [providerAccountKeypair, acc1].map((keypair) =>
    getAssociatedTokenAddressSync(token1.publicKey, keypair.publicKey, false, TOKEN_PROGRAM_ID),
);
// const otherWalletPubkey = new PublicKey('FzN8SmLZPXmBSSv5QnyTgJUYGajHycZZd6xdDqk1igfJ');
// const tokenAccount2 = getAssociatedTokenAddressSync(token1.publicKey, otherWalletPubkey, false, TOKEN_PROGRAM_ID);

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const denomination = new BN(10).pow(new BN(9));
const feeL2Percentage = 0;
const feeTreasuryPercentage = 100_00;
const minerMinimumStake = new BN("25000").mul(denomination);
const minerRequirement = 3;
const blockPerEpoch = new BN(600 * 2);
const epochDuration = new BN(1200 * 0.4);
const rewardPerEpoch = new BN("38").mul(denomination).div(new BN("100"));
const submitDuration = new BN(10 * 6 * 5);
const commitDuration = new BN(10 * 6 * 5);
const revealDuration = new BN(10 * 6 * 5);
const unstakeDelayTime = new BN(1814400); // NOTE:  1,814,400 blocks = 21 days
const penaltyDuration = new BN(1200); // NOTE: 3.3 hours
const finePercentage = 10_00;
const feeRatioMinerValidator = 50_00; // Miner earns 50% of the workers fee ( = [msg.value - L2's owner fee - treasury] )
const minFeeToUse = new BN("10").mul(denomination).div(new BN("100")); // 0.1 SOL
const daoTokenReward = new BN("0");
const daoTokenPercentage = {
    minerPercentage: 50_00,
    userPercentage: 30_00,
    referrerPercentage: 5_00,
    refereePercentage: 5_00,
    l2OwnerPercentage: 10_00,
};
const zeroValue = 0;
export const init = async () => {
    const minimumLamports = await getMinimumBalanceForRentExemptMint(provider.connection);
    const sendSolInstructions: Array<TransactionInstruction> = [acc1].map((account) =>
    SystemProgram.transfer({
      fromPubkey: providerAccountKeypair.publicKey,
      toPubkey: account.publicKey,
      lamports: 0.01 * LAMPORTS_PER_SOL,
    }),
  );
    const createMintInstructions: Array<TransactionInstruction> = [
        SystemProgram.createAccount({
            fromPubkey: providerAccountKeypair.publicKey,
            newAccountPubkey: token1.publicKey,
            lamports: minimumLamports,
            space: MINT_SIZE,
            programId: TOKEN_PROGRAM_ID,
        }),
    ];
    const mintTokensInstructions: Array<TransactionInstruction> = [
        {
            mint: token1.publicKey,
            authority: acc1.publicKey,
            ata: [{ ata: tokenAccount0, owner: providerAccountKeypair.publicKey }, { ata: tokenAccount1, owner: acc1.publicKey }],
            // ata: [{ ata: tokenAccount2, owner: otherWalletPubkey }],
        },
    ].flatMap((mintDetails) => [
        createInitializeMint2Instruction(mintDetails.mint, 6, mintDetails.authority, null, TOKEN_PROGRAM_ID),
        ...mintDetails.ata.map(d => createAssociatedTokenAccountIdempotentInstruction(providerAccountKeypair.publicKey, d.ata, d.owner, mintDetails.mint, TOKEN_PROGRAM_ID)),
        ...mintDetails.ata.map(d => createMintToInstruction(mintDetails.mint, d.ata, mintDetails.authority, 1_000_000_000, [], TOKEN_PROGRAM_ID)),
    ]);
    // const tx0 = new Transaction();
    // tx0.instructions = [...sendSolInstructions, ...createMintInstructions, ...mintTokensInstructions];
    // tx0.instructions = [...mintTokensInstructions];
    // console.log('signers', [token1, providerAccountKeypair, acc1].map(k => k.publicKey.toBase58()));
    // await provider.sendAndConfirm(tx0, [providerAccountKeypair, acc1]);
    // console.log("tx0", tx0);
    const seeds = [Buffer.from("model_storage"), id.toBuffer("le", 8)];
    const [modelStorage, bump] = await PublicKey.findProgramAddress(seeds, solearnProgram.programId);
    const accounts: Record<string, PublicKey> = {
        tokenProgram: TOKEN_PROGRAM_ID,
        sysvarClock: SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
    };
    const stakingTokenPubKey = new PublicKey(token1.publicKey.toBase58()); // new PublicKey("6qKcWsgczLGbpDH6426r4govbd1HgDMg6j5LLikx3bop");
    accounts.admin = providerAccountKeypair.publicKey;
    accounts.stakingToken = stakingTokenPubKey;
    accounts.solLearnAccount = solLearnAccount.publicKey;

    accounts.vaultWalletOwnerPda = PublicKey.findProgramAddressSync(
        [Buffer.from('vault'), solLearnAccount.publicKey.toBuffer()],
        solearnProgram.programId,
    )[0];

    accounts.models = PublicKey.findProgramAddressSync(
        [Buffer.from('models'), solLearnAccount.publicKey.toBuffer()],
        solearnProgram.programId,
    )[0];
    accounts.miner = providerAccountKeypair.publicKey;
    accounts.minerAccount = PublicKey.findProgramAddressSync(
        [
            Buffer.from("miner"),
            providerAccountKeypair.publicKey.toBuffer(),
            solLearnAccount.publicKey.toBuffer(),
        ],
        solearnProgram.programId
    )[0];

    accounts.minersOfModel = PublicKey.findProgramAddressSync(
        [
            Buffer.from("models"),
            solLearnAccount.publicKey.toBuffer(),
            model1.publicKey.toBuffer(),
        ],
        solearnProgram.programId
    )[0];
    accounts.minerStakingWallet = tokenAccount1;

    // const tx = await solearnProgram.methods
    //     .initialize(
    //         // minerMinimumStake,
    //         // rewardPerEpoch,
    //         // epochDuration,
    //         // wallet.publicKey, // treasury_address
    //         // feeL2Percentage,
    //         // feeTreasuryPercentage,
    //         // feeRatioMinerValidator,
    //         // submitDuration,
    //         // commitDuration,
    //         // revealDuration,
    //         // penaltyDuration,
    //         // minerRequirement,
    //         // blockPerEpoch,
    //         // finePercentage,
    //         // daoTokenReward,
    //         // daoTokenPercentage.minerPercentage,
    //         // daoTokenPercentage.userPercentage,
    //         // daoTokenPercentage.referrerPercentage,
    //         // daoTokenPercentage.refereePercentage,
    //         // daoTokenPercentage.l2OwnerPercentage,
    //         // unstakeDelayTime
    //         new BN(1000000),
    //         new BN(100000000), // EAI decimal 6  => 10 EAI
    //         new BN(10), // 10 blocks
    //         treasury.publicKey, // treasury address
    //         100, // 1% 
    //         100, // 1% 
    //         5, // dont know this value
    //         new BN(10), // 10s
    //         new BN(10), // 10s
    //         new BN(10), // 10s
    //         new BN(10), // 10s
    //         1,
    //         new BN(10), // 10 blocks
    //         100,
    //         new BN(0),
    //         zeroValue, zeroValue, zeroValue, zeroValue, zeroValue, 
    //         new BN(10), // 10s unstaking
    //     )
    //     .accounts(accounts)
    //     .signers([providerAccountKeypair, solLearnAccount])
    //     .rpc();
    // console.log("tx", tx);
    // await sleep(1000);

    // const tx2 = await solearnProgram.methods
    //     .initialize2()
    //     .accounts(accounts)
    //     .rpc();
    // console.log("tx2", tx2);
    // await sleep(1000);

    const tx3 = await solearnProgram.methods
        .addModel(model1.publicKey)
        .accounts(accounts)
        .rpc();
    console.log("tx3", tx3);
    await sleep(1000);

    // const tx4 = await solearnProgram.methods
    //     .joinForMinting()
    //     .accounts(accounts)
    //     .signers([providerAccountKeypair])
    //     .rpc();
    // console.log("tx3", tx4);
    // await sleep(1000);
}
init();

import { AnchorProvider, Program, Wallet, setProvider } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import type { Solearn } from "../target/types/solearn";
import type { BasicToken } from "../target/types/basic_token";
import solearnIdl from "../target/idl/solearn.json";
import tokenIdl from "../target/idl/basic_token.json";
import { BN } from "bn.js";

const PRIVKEY_BYTES = [];

const newAccountKp = Keypair.fromSecretKey(new Uint8Array(PRIVKEY_BYTES));
// console.log("newAccountKp", newAccountKp);
const url = "http://127.0.0.1:8899"; // clusterApiUrl("devnet")
const connection = new Connection(url, "confirmed");
const wallet = new Wallet(newAccountKp);
console.log("wallet", wallet.publicKey.toBase58());

const provider = new AnchorProvider(connection, wallet, {});
setProvider(provider);
 
export const solearnProgram = new Program(solearnIdl as Solearn, provider);
export const tokenProgram = new Program(tokenIdl as BasicToken, provider);
console.log("Program", solearnProgram.idl.address, "Token Program", tokenProgram.programId);
let id = new BN(1);

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
export const init = async () => {
    const seeds = [Buffer.from("model_storage"), id.toBuffer("le", 8)];
    const [modelStorage, bump] = await PublicKey.findProgramAddress(seeds, solearnProgram.programId);

    const transactionSignature = await solearnProgram.methods
        .initialize(
            minerMinimumStake,
            rewardPerEpoch,
            epochDuration,
            wallet.publicKey, // treasury_address
            feeL2Percentage,
            feeTreasuryPercentage,
            feeRatioMinerValidator,
            submitDuration,
            commitDuration,
            revealDuration,
            penaltyDuration,
            minerRequirement,
            blockPerEpoch,
            finePercentage,
            daoTokenReward,
            daoTokenPercentage.minerPercentage,
            daoTokenPercentage.userPercentage,
            daoTokenPercentage.referrerPercentage,
            daoTokenPercentage.refereePercentage,
            daoTokenPercentage.l2OwnerPercentage,
        )
        .accounts({
            admin: wallet.publicKey,
            stakingToken: tokenProgram.programId,
        })
        .signers([newAccountKp])
        .rpc();
    console.log("transactionSignature", transactionSignature);
}
init();

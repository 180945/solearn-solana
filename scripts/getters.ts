import { AnchorProvider, Program, Wallet, setProvider } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import type { Solearn } from "../target/types/solearn";
import solearnIdl from "../target/idl/solearn.json";
import { BN } from "bn.js";

const PRIVKEY_BYTES = [];

const providerAccountKeypair = Keypair.fromSecretKey(new Uint8Array(PRIVKEY_BYTES));
const url = clusterApiUrl("devnet")
const connection = new Connection(url, "confirmed");
const wallet = new Wallet(providerAccountKeypair);

const provider = new AnchorProvider(connection, wallet, {});
setProvider(provider);
 
export const program = new Program(solearnIdl as Solearn, provider);
console.log("Program", program.idl.address);


export const fetchInference = async () => {
    let infId = new BN(1);
    const seeds = [Buffer.from('inference'), infId.toBuffer('le', 8)];
    const [infs, bump] = await PublicKey.findProgramAddress(seeds, program.programId);
    const fetched = await program.account.inference.fetch(infs);
    console.log("Fetched inference", fetched);
    return fetched;
}

export const fetchAgentInfo = async () => {
    const minerPubkey = new PublicKey('FEkiyRzejZbVE9oz449o1kJBEjXwV2dExbcQdLX6hoH6');
    const solearnPubkey = new PublicKey('4xhkpgjFuZd9rvCxRPbjmy8DvkK9D4YXg5554Tt22Wa6');
    const seeds = [Buffer.from('miner'), minerPubkey.toBuffer(), solearnPubkey.toBuffer()];
    const [minerAccount, bump] = await PublicKey.findProgramAddress(seeds, program.programId);
    const fetched = await program.account.minerInfo.fetch(minerAccount);
    console.log("Fetched agent", fetched);
    return fetched;
}

fetchAgentInfo();
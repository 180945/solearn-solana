import { AnchorProvider, Program, Wallet, setProvider } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import type { HybridModel } from "../target/types/hybrid_model";
import hm from "../target/idl/hybrid_model.json";
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
 
export const program = new Program(hm as HybridModel, provider);
console.log("Program", program.idl.address);
let dummy_pubkey = program.programId; // TODO
let id = new BN(1);

export const init = async () => {
    const seeds = [Buffer.from("model_storage"), id.toBuffer("le", 8)];
    const [modelStorage, bump] = await PublicKey.findProgramAddress(seeds, program.programId);
    const transactionSignature = await program.methods
        .initialize(id,
            "hmodel1",
            "some metadata",
            dummy_pubkey,
            dummy_pubkey)
        .accounts({
            admin: wallet.publicKey,
        })
        .signers([newAccountKp])
        .rpc();
    console.log("transactionSignature", transactionSignature);
}
init();
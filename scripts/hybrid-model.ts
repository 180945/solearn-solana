import { AnchorProvider, Program, Wallet, setProvider } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection, PublicKey, Keypair, SystemProgram, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import type { HybridModel } from "../target/types/hybrid_model";
import hm from "../target/idl/hybrid_model.json";
import { BN } from "bn.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

const PRIVKEY_BYTES = [];

const newAccountKp = Keypair.fromSecretKey(new Uint8Array(PRIVKEY_BYTES));
// console.log("newAccountKp", newAccountKp);
const url = clusterApiUrl("devnet")
const connection = new Connection(url, "confirmed");
const wallet = new Wallet(newAccountKp);
console.log("wallet", wallet.publicKey.toBase58());

const provider = new AnchorProvider(connection, wallet, {});
setProvider(provider);
 
export const program = new Program(hm as HybridModel, provider);
console.log("Program", program.idl.address);
let dummy_keypairs = [Keypair.generate(), Keypair.generate()];
let id = new BN(990001);

export const init = async () => {
    const seeds = [Buffer.from("model_storage"), id.toBuffer('le', 8)];
    const accounts: Record<string, PublicKey> = {
        sysvarClock: SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
    };
    accounts.modelStorage = PublicKey.findProgramAddressSync(seeds, program.programId)[0];
    accounts.admin = wallet.publicKey;
    const transactionSignature = await program.methods
        .initialize(id,
            "hmodel1",
            "some metadata",
            dummy_keypairs[0].publicKey,
            dummy_keypairs[1].publicKey,
        )
        .accounts({ ...accounts })
        .signers([newAccountKp])
        .rpc();
    console.log("transactionSignature", transactionSignature);
}
init();
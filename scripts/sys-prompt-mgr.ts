import { AnchorProvider, Program, Wallet, setProvider } from "@coral-xyz/anchor";
import { clusterApiUrl, Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import type { NftProgram } from "../target/types/nft_program";
import p from "../target/idl/nft_program.json";
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
 
export const program = new Program(p as NftProgram, provider);
console.log("Program", program.idl.address);
let dummy_pubkey = program.programId; // TODO
let id = new BN(1);

export const createNft = async () => {
    const seeds = [Buffer.from("model_storage"), id.toBuffer("le", 8)];
    const [modelStorage, bump] = await PublicKey.findProgramAddress(seeds, program.programId);
    const transactionSignature = await program.methods
        .createSingleNft(id,
            "nftoken1",
            "nft1",
            "uri://example.com"
        )
        .accounts({
            authority: wallet.publicKey,
            payer: wallet.publicKey,
        })
        .signers([newAccountKp])
        .rpc();
    console.log("transactionSignature", transactionSignature);
}
createNft();
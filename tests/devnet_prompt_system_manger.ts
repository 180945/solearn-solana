import { describe, it } from 'node:test';
import * as anchor from '@coral-xyz/anchor';
import { BN, type Program } from '@coral-xyz/anchor';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from '@solana/spl-token';
import { Keypair, SYSVAR_CLOCK_PUBKEY, SYSVAR_RENT_PUBKEY, PublicKey } from '@solana/web3.js';
const TOKEN_PROGRAM = TOKEN_PROGRAM_ID;
const IDL = require('../target/idl/prompt_system_manager.json');
IDL.address = new PublicKey("8CgzLBj4wq4pwKMv52BGnhaJLE22LEsv7obNTJtASNps");
const METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

import { PromptSystemManager } from "../target/types/prompt_system_manager";
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';

describe('Prompt System Manager Bankrun test', () => {
  let context, provider, connection, program;
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
    program = new anchor.Program<PromptSystemManager>(IDL, anchor.AnchorProvider.env());
    console.log(program);
    const programProvider = program.provider as anchor.AnchorProvider;
    // const minimumLamports = await getMinimumBalanceForRentExemptMint(connection);
    const adminImportded = programProvider.wallet;
    const admin = Keypair.fromSecretKey((adminImportded as NodeWallet).payer.secretKey);

    // START ADD CODE HERE
    let collection_id = new BN(2);

    accounts.authority = admin.publicKey;
    accounts.payer = admin.publicKey;
    accounts.mint = PublicKey.findProgramAddressSync(
      [Buffer.from('mint'), collection_id.toArrayLike(Buffer, 'le', 8)],
      program.programId,
    )[0];
    accounts.tokenAccount = getAssociatedTokenAddressSync(accounts.mint, accounts.authority, false, TOKEN_PROGRAM);
    accounts.masterEditionAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer(), Buffer.from('edition')],
      accounts.metadataProgram,
    )[0];
    accounts.nftMetadata = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer()],
      accounts.metadataProgram,
    )[0];

    // const tx = await program.methods
    //     .createSingleNft(
    //         collection_id,
    //         "test",
    //         "T",
    //         "test",
    //     )
    //     .accounts({
    //         ...accounts
    //     })
    //     .signers([admin])
    //     .rpc();

    // console.log({tx});


    let nft_id = new BN(1);
    accounts.collection = accounts.tokenAccount;
    accounts.collectionMint = accounts.mint;
    accounts.mint = PublicKey.findProgramAddressSync(
      [Buffer.from('mint'), collection_id.toArrayLike(Buffer, 'le', 8), nft_id.toArrayLike(Buffer, 'le', 8)],
      program.programId,
    )[0];

    accounts.tokenAccount = getAssociatedTokenAddressSync(accounts.mint, accounts.authority, false, TOKEN_PROGRAM);
    accounts.masterEditionAccount = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer(), Buffer.from('edition')],
      accounts.metadataProgram,
    )[0];
    accounts.nftMetadata = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), accounts.metadataProgram.toBuffer(), accounts.mint.toBuffer()],
      accounts.metadataProgram,
    )[0];

    // await sendAndConfirmTx(provider, [await program.instruction.mintToCollection(
    //     collection_id,
    //     nft_id,
    //     "test",
    //     "T",
    //     "test",
    //     {
    //       accounts: {...accounts}
    //     }
    //   )], [alice]);

    const tx = await program.methods.mintToCollection
    (
        collection_id,
        nft_id,
        "test",
        "T",
        "test",
    )
    .accounts({
        ...accounts
    })
    .signers([admin])
    .rpc();

    console.log({tx});
    
  }

  it('Devnet Create an NFT!', async () => {
    await initProgram();
  });
});
use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hash;

pub fn random_number(clk: &Clock, nonce: u64, range: u64) -> u64 {
    if range == 0 {
        return 0;
    }

    let mut cloned_data: Vec<u8> = vec![];
    let nonce_bytes = nonce.to_le_bytes();
    cloned_data.extend_from_slice(&nonce_bytes);
    let time_bytes = (clk.unix_timestamp as u64) as u64;
    cloned_data.extend_from_slice(&time_bytes.to_le_bytes());
    hash(&cloned_data);
    let rightmost: &[u8] = &cloned_data[28..];

    let seed = u64::try_from_slice(rightmost).unwrap();

    seed % range
}

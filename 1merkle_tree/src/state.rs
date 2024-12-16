use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::hash:: Hash;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct CounterAccount {
    pub count: i64,
    pub root_hash: Hash,
    pub leafs: Vec<String> ,
}

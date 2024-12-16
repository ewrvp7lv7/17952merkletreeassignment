use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};

declare_id!("4ekk1PnQEv3ak6kv88UChc1uc7769FgMNgdgT5h5m3qB");

pub const MAX_LEAVES: usize = 256;
pub const HASH_SIZE: usize = 32;

#[program]
mod merkle_tree {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let merkle_account = &mut ctx.accounts.merkle_account;
        merkle_account.root = [0; HASH_SIZE];
        merkle_account.leaf_count = 0;
        merkle_account.leaves = Vec::with_capacity(MAX_LEAVES);
        Ok(())
    }

    pub fn insert_leaf(ctx: Context<InsertLeaf>, leaf: [u8; HASH_SIZE]) -> Result<()> {
        let merkle_account = &mut ctx.accounts.merkle_account;

        require!(
            merkle_account.leaf_count < MAX_LEAVES as u32,
            MerkleError::TreeFull
        );

        merkle_account.leaves.push(leaf);
        merkle_account.leaf_count += 1;
        merkle_account.root = calculate_merkle_root(&merkle_account.leaves);

        emit!(LeafInserted {
            leaf,
            index: merkle_account.leaf_count - 1,
            root: merkle_account.root
        });
        msg!("Лист добавлен: индекс={}, корень={:?}", merkle_account.leaf_count - 1, merkle_account.root);

        Ok(())
    }

    pub fn verify_proof(
        ctx: Context<VerifyProof>,
        leaf: [u8; HASH_SIZE],
        proof: Vec<[u8; HASH_SIZE]>,
        path: Vec<bool>,
    ) -> Result<bool> {
        require!(proof.len() == path.len(), MerkleError::InvalidProof);
        
        let calculated_root = calculate_proof_root(leaf, &proof, &path);
        let is_valid = calculated_root == ctx.accounts.merkle_account.root;
        
        msg!("Проверка доказательства: результат={}, вычисленный_корень={:?}", is_valid, calculated_root);
        
        Ok(is_valid)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + HASH_SIZE + 4 + (MAX_LEAVES * HASH_SIZE)
    )]
    pub merkle_account: Account<'info, MerkleAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InsertLeaf<'info> {
    #[account(mut)]
    pub merkle_account: Account<'info, MerkleAccount>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyProof<'info> {
    pub merkle_account: Account<'info, MerkleAccount>,
}

#[account]
pub struct MerkleAccount {
    pub root: [u8; HASH_SIZE],
    pub leaf_count: u32,
    pub leaves: Vec<[u8; HASH_SIZE]>,
}

#[event]
pub struct LeafInserted {
    pub leaf: [u8; HASH_SIZE],
    pub index: u32,
    pub root: [u8; HASH_SIZE],
}

#[error_code]
pub enum MerkleError {
    #[msg("Дерево Меркла заполнено")]
    TreeFull,
    #[msg("Недопустимый лист (нулевой)")]
    InvalidLeaf,
    #[msg("Недопустимое доказательство")]
    InvalidProof,
}

fn calculate_merkle_root(leaves: &Vec<[u8; HASH_SIZE]>) -> [u8; HASH_SIZE] {
    if leaves.is_empty() {
        return [0; HASH_SIZE];
    }

    let mut current_level = leaves.clone();
    
    while current_level.len() > 1 {
        current_level = current_level
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    let mut hasher = Sha256::new();
                    hasher.update(&chunk[0]);
                    hasher.update(&chunk[1]);
                    let result = hasher.finalize();
                    let mut hash_bytes = [0u8; HASH_SIZE];
                    hash_bytes.copy_from_slice(&result);
                    hash_bytes
                } else {
                    chunk[0]
                }
            })
            .collect();
    }

    current_level[0]
}

fn calculate_proof_root(
    leaf: [u8; HASH_SIZE],
    proof: &Vec<[u8; HASH_SIZE]>,
    path: &Vec<bool>,
) -> [u8; HASH_SIZE] {
    let mut current = leaf;
    
    for (proof_element, is_left) in proof.iter().zip(path.iter()) {
        let mut hasher = Sha256::new();
        if *is_left {
            hasher.update(proof_element);
            hasher.update(&current);
        } else {
            hasher.update(&current);
            hasher.update(proof_element);
        }
        let result = hasher.finalize();
        current.copy_from_slice(&result);
    }
    
    current
}
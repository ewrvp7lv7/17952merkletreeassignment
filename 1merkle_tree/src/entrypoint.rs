use instructions::{CounterInstruction, Unpack};
use processor::*;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use solana_program::{entrypoint, msg};

pub mod error;
pub mod instructions;
pub mod merkle_tree;
pub mod processor;
pub mod state;

entrypoint!(entrypoints);
pub fn entrypoints(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("pub fn entrypoints {}", instruction_data[0]);
    let instruction = CounterInstruction::unpack(instruction_data)?;
    match instruction {
        CounterInstruction::InitCounter(init_val) => {
            process_initialize_counter(program_id, accounts, init_val)
        }
        CounterInstruction::IncCounter => process_change_counter(program_id, accounts, true),
        CounterInstruction::DecCounter => process_change_counter(program_id, accounts, false),
        CounterInstruction::InitTree(leaf) => init_tree(program_id, accounts, leaf),
    }
}

#[cfg(test)]
mod tests;

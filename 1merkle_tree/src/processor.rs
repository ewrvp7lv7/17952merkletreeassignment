use crate::{error::CustomError, merkle_tree::MerkleTree, state::CounterAccount};
use borsh::BorshSerialize;
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
    system_instruction, sysvar::Sysvar,
};

pub fn process_initialize_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    initial_value: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let counter_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    msg!("system_program {}", system_program.key);
    // let size_data = i64::BITS / 8;
    let size_data = 10u64 * 1024;

    let rent = Rent::get()?;

    let required_lamports = rent.minimum_balance(size_data as usize);
    msg!("minimum_balance {}", required_lamports);

    // invoke(
    //     &system_instruction::create_account(
    //         payer_account.key,   //account paying for the new account
    //         counter_account.key, //account to be created
    //         required_lamports,   // amount of lamport given to the new account
    //         size_data,           //size in bytes to allocate for the data field
    //         program_id,          //program owner is set to our program
    //     ),
    //     &[
    //         payer_account.clone(),
    //         counter_account.clone(),
    //         system_program.clone(),
    //     ],
    // )?;

    assert!(payer_account.is_writable);
    assert!(payer_account.is_signer);
    assert!(counter_account.is_writable);
    assert_eq!(counter_account.owner, &solana_program::system_program::ID); //владелец 111 то есть никто
    assert!(solana_program::system_program::check_id(system_program.key));

    let vault_bump_seed = initial_value;
    let vault_seeds = &[b"vault", payer_account.key.as_ref(), &[vault_bump_seed]];
    let expected_vault_pda = Pubkey::create_program_address(vault_seeds, program_id)?;

    assert_eq!(counter_account.key, &expected_vault_pda);

    invoke_signed(
        &system_instruction::create_account(
            payer_account.key,   //account paying for the new account
            counter_account.key, //account to be created
            required_lamports,   // amount of lamport given to the new account
            size_data.into(),    //size in bytes to allocate for the data field
            program_id,          //program owner is set to our program
        ),
        &[
            payer_account.clone(),
            counter_account.clone(),
            system_program.clone(),
        ],
        // A slice of seed slices, each seed slice being the set
        // of seeds used to generate one of the PDAs required by the
        // callee program, the final seed being a single-element slice
        // containing the `u8` bump seed.
        &[&[b"vault", payer_account.key.as_ref(), &[initial_value]]],
    )?;

    let counter_data = CounterAccount {
        count: initial_value.into(),
        root_hash: [1; 32].into(),
        leafs: vec!["args".to_string(), "args".to_string(), "args!".to_string()],
    };

    let mut account_data = &mut counter_account.data.borrow_mut()[..];
    counter_data.serialize(&mut account_data)?;
    msg!("counter init to {}", initial_value);
    Ok(())
}

pub fn process_change_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    inc: bool,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let counter_account = next_account_info(accounts_iter)?;

    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = counter_account.data.borrow_mut();
    // let data = counter_account.data.borrow();
    // let mut counter_data: CounterAccount = CounterAccount::try_from_slice(&data)?;
    let mut counter_data =
        solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&data).unwrap();
    match inc {
        true => {
            counter_data.count = counter_data
                .count
                .checked_add(1)
                .ok_or(CustomError::Overflow)?;
        }
        false => {
            counter_data.count = counter_data
                .count
                .checked_sub(1)
                .ok_or(CustomError::Underflow)?;
        }
    };

    counter_data.serialize(&mut &mut data[..])?;
    msg!("counter incremented to: {}", counter_data.count);
    Ok(())
}

pub fn init_tree(program_id: &Pubkey, accounts: &[AccountInfo], leaf: String) -> ProgramResult {
    msg!("init_tree: {}", leaf);

    let accounts_iter = &mut accounts.iter();

    let counter_account = next_account_info(accounts_iter)?;

    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = counter_account.data.borrow_mut();
    let mut counter_data =
        solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&data).unwrap();

    counter_data.leafs.push(leaf);

    let tree = MerkleTree::new(&counter_data.leafs);

    counter_data.root_hash = *tree.get_root().unwrap();

    counter_data.serialize(&mut &mut data[..])?;
    msg!("root_hash: {}", counter_data.root_hash);

    Ok(())
}

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};

use merkle_tree_program::instructions::CounterInstruction;
use merkle_tree_program::state::CounterAccount;

#[tokio::main]
async fn main() {
    let keypair = read_keypair_file("./target/deploy/merkle_tree_program-keypair.json").unwrap();
    let program_id = keypair.pubkey();

    // Connect to the Solana devnet
    let rpc_url = String::from("http://127.0.0.1:8899");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // Generate a new keypair for the payer
    let payer = Keypair::new();

    // Request airdrop
    let airdrop_amount = 1_000_000_000; // 1 SOL
    let signature = client
        .request_airdrop(&payer.pubkey(), airdrop_amount)
        .expect("Failed to request airdrop");

    // Wait for airdrop confirmation
    loop {
        let confirmed = client.confirm_transaction(&signature).unwrap();
        if confirmed {
            break;
        }
    }

    // let counter_keypair = Keypair::new();

    let (counter_keypair_pub, bump_seed) =
        Pubkey::find_program_address(&[b"vault", payer.pubkey().as_ref()], &program_id);

    println!("Testing init...");

    // let init_val = 10i64;
    let init_val: i64 = bump_seed.into();

    let data = CounterInstruction::InitCounter(bump_seed);

    let init_instruction = Instruction::new_with_borsh(
        program_id,
        &data,
        vec![
            // AccountMeta::new(counter_keypair_pub, true),
            AccountMeta::new(counter_keypair_pub, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // Add the instruction to new transaction
    let mut tx = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer], client.get_latest_blockhash().unwrap());

    // Send and confirm the transaction
    match client.send_and_confirm_transaction(&tx) {
        Ok(signature) => println!("Success Init Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }

    let account = match client.get_account(&counter_keypair_pub) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("failed to get counter account: {}", err);
            panic!("{}", err)
        }
    };
    let counter =
        solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&account.data).unwrap();
    assert_eq!(counter.count, init_val);
    println!("counter init successfully with value {}", counter.count);
}

// let keypair = read_keypair_file("./target/deploy/counter_program-keypair.json").unwrap();
// let program_id = keypair.pubkey();

// // Connect to the Solana devnet
// let rpc_url = String::from("http://127.0.0.1:8899");
// let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

// // Generate a new keypair for the payer
// let payer = Keypair::new();

// // Request airdrop
// let airdrop_amount = 1_000_000_000; // 1 SOL
// let signature = client
//     .request_airdrop(&payer.pubkey(), airdrop_amount)
//     .expect("Failed to request airdrop");

// // Wait for airdrop confirmation
// loop {
//     let confirmed = client.confirm_transaction(&signature).unwrap();
//     if confirmed {
//         break;
//     }
// }

// // let counter_keypair = Keypair::new();

// let (counter_keypair_pub, bump_seed) =
//     Pubkey::find_program_address(&[b"vault", payer.pubkey().as_ref()], &program_id);
// let signers = &[&payer];

// println!("Testing init...");

// // let init_val = 10i64;
// let init_val: i64 = bump_seed.into();

// let data = CounterInstruction::InitCounter(bump_seed);

// let init_instruction = Instruction::new_with_borsh(
//     program_id,
//     &data,
//     vec![
//         // AccountMeta::new(counter_keypair_pub, true),
//         AccountMeta::new(counter_keypair_pub, false),
//         AccountMeta::new(payer.pubkey(), true),
//         AccountMeta::new_readonly(system_program::id(), false),
//     ],
// );

// // let tx_init = CounterInstruction::InitCounter;
// // // Create the instruction
// // let ix = Instruction::new_with_borsh(
// //     program_id,
// //     &tx_init,
// //     vec![
// //         AccountMeta::new(counter_keypair.pubkey(), true),
// //         AccountMeta::new(payer.pubkey(), true),
// //         AccountMeta::new(system_program::id(), false),
// //     ],
// // );

// // Add the instruction to new transaction
// let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
// tx.sign(
//     &[&payer, &counter_keypair],
//     client.get_latest_blockhash().unwrap(),
// );

// // Send and confirm the transaction
// match client.send_and_confirm_transaction(&tx) {
//     Ok(signature) => println!("Success Init Transaction Signature: {}", signature),
//     Err(err) => eprintln!("Error sending transaction: {}", err),
// }

// let tx_inc = CounterInstruction::IncCounter;

// let ix = Instruction::new_with_borsh(
//     program_id,
//     &tx_inc,
//     vec![AccountMeta::new(counter_keypair.pubkey(), true)],
// );
// let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
// tx.sign(
//     &[&payer, &counter_keypair],
//     client.get_latest_blockhash().unwrap(),
// );

// // Send and confirm the transaction
// match client.send_and_confirm_transaction(&tx) {
//     Ok(signature) => println!("Success Inc Transaction Signature: {}", signature),
//     Err(err) => eprintln!("Error sending transaction: {}", err),
// }
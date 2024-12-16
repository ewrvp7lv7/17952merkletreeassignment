// use borsh::BorshDeserialize;
use env_logger;
use instructions::CounterInstruction;
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    // signature::Keypair,
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use state::CounterAccount;

use super::*;

fn panic_log(content: String) -> ! {
    eprintln!("{}", content);
    panic!("{}", content);
}

fn setup() {
    env_logger::builder()
        .is_test(true)
        //disable all logs by defaykt
        .filter(None, log::LevelFilter::Off)
        //enable solana program logs
        .filter(
            Some("solana_runtime::message_processor::stable_log"),
            log::LevelFilter::Trace,
        )
        //enable our test log
        .filter_module("counter_program::test", log::LevelFilter::Trace)
        .init();
}

#[tokio::test]
async fn test_init() {
    setup();

    let program_id = Pubkey::new_unique();
    let (mut bank_clients, payer, recent_blockhash) =
        ProgramTest::new(env!("CARGO_PKG_NAME"), program_id, processor!(entrypoints))
            .start()
            .await;

    let account = match bank_clients.get_account(payer.pubkey()).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        println!("Balance:{}", account_data.lamports);
    } else {
        panic_log("No counter account found".to_string());
    }

    // let counter_keypair = Keypair::new();
    // let counter_keypair_pub = counter_keypair.pubkey();
    // let bump_seed = 128u8;
    // let signers =&[&payer, &counter_keypair];

    let (counter_keypair_pub, bump_seed) =
        Pubkey::find_program_address(&[b"vault", payer.pubkey().as_ref()], &program_id);
    let signers = &[&payer];

    println!("Testing init...");

    // let init_val = 10i64;
    let init_val: i64 = bump_seed.into();

    // let mut init_instruction_data: Vec<u8> = vec![CounterInstruction::InitCounter as u8];
    // init_instruction_data.extend_from_slice(&init_val.to_le_bytes());

    // let init_instruction = Instruction::new_with_bytes(
    //     program_id,
    //     &init_instruction_data,
    //     vec![
    //         AccountMeta::new(counter_keypair_pub, true),
    //         AccountMeta::new(payer.pubkey(), true),
    //         AccountMeta::new_readonly(system_program::id(), false),
    //     ],
    // );

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
    // println!("Test1");
    let mut tx = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    tx.sign(signers, recent_blockhash);

    // // tx.partial_sign(signers, recent_blockhash);
    // let tx = Transaction::new_signed_with_payer(
    //     &[init_instruction],
    //     Some(&payer.pubkey()),
    //     &[&payer], //&[&payer, &counter_keypair];
    //     recent_blockhash,
    // );

    bank_clients.process_transaction(tx).await.unwrap();
    // println!("Test2");

    let account = match bank_clients.get_account(counter_keypair_pub).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        // let counter = CounterAccount::try_from_slice(&account_data.data)
        //     .expect("failed to deserialize counter data"); //глючит Custom { kind: InvalidData, error: "Not all bytes read" }
        let counter =
            solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&account_data.data)
                .unwrap();
        assert_eq!(counter.count, init_val);
        println!("counter init successfully with value {}", counter.count);
    } else {
        panic_log("No counter account found".to_string());
    }

    println!("Testing counter increment...");

    // let inc_instructionb = Instruction::new_with_bytes(
    //     program_id,
    //     &[CounterInstruction::IncCounter as u8],
    //     vec![AccountMeta::new(counter_keypair_pub, true)],
    // );

    let data = CounterInstruction::IncCounter;

    let inc_instructionb = Instruction::new_with_borsh(
        program_id,
        &data,
        vec![
            // AccountMeta::new(counter_keypair_pub, true),
            AccountMeta::new(counter_keypair_pub, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[inc_instructionb], Some(&payer.pubkey()));
    tx.sign(signers, recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    let account = match bank_clients.get_account(counter_keypair_pub).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        // let counter = CounterAccount::try_from_slice(&account_data.data)
        //     .expect("failed to deserialize counter data");
        let counter =
            solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&account_data.data)
                .unwrap();
        assert_eq!(counter.count, init_val + 1);
        println!("counter incremented successfully to {}", counter.count);
    } else {
        panic_log(format!("No counter account found"));
    }

    println!("Testing counter decrement...");

    // let dec_instruction = Instruction::new_with_bytes(
    //     program_id,
    //     &[CounterInstruction::DecCounter as u8],
    //     vec![AccountMeta::new(counter_keypair_pub, true)],
    // );

    let data = CounterInstruction::DecCounter;

    let dec_instruction = Instruction::new_with_borsh(
        program_id,
        &data,
        vec![
            // AccountMeta::new(counter_keypair_pub, true),
            AccountMeta::new(counter_keypair_pub, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[dec_instruction], Some(&payer.pubkey()));
    tx.sign(signers, recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    let account = match bank_clients.get_account(counter_keypair_pub).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        // let counter = CounterAccount::try_from_slice(&account_data.data)
        //     .expect("failed to deserialize counter data");
        let counter =
            solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&account_data.data)
                .unwrap();
        assert_eq!(counter.count, init_val);
        println!("counter decremented successfully to {}", counter.count);
    } else {
        panic_log(format!("No counter account found"));
    }

    println!("Testing Strings..");

    // let dec_instruction = Instruction::new_with_bytes(
    //     program_id,
    //     &[CounterInstruction::DecCounter as u8],
    //     vec![AccountMeta::new(counter_keypair_pub, true)],
    // );

    let data = CounterInstruction::InitTree("Test-string1".to_string());

    let dec_instruction = Instruction::new_with_borsh(
        program_id,
        &data,
        vec![
            // AccountMeta::new(counter_keypair_pub, true),
            AccountMeta::new(counter_keypair_pub, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[dec_instruction], Some(&payer.pubkey()));
    tx.sign(signers, recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    let account = match bank_clients.get_account(counter_keypair_pub).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        let mut counter_data =
            solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&account_data.data)
                .unwrap();

        if let CounterInstruction::InitTree(init_str) = data {
            println!("root_hash {}", counter_data.root_hash);
            println!("leafs {:?}", counter_data.leafs);

            counter_data.leafs.pop();
            counter_data.leafs.push(init_str);

            let tree = merkle_tree::MerkleTree::new(&counter_data.leafs);

            let new_hash = *tree.get_root().unwrap();
            assert_eq!(counter_data.root_hash, new_hash);
        }
    } else {
        panic_log(format!("No counter account found"));
    }

    println!("Testing Strings..");

    // let dec_instruction = Instruction::new_with_bytes(
    //     program_id,
    //     &[CounterInstruction::DecCounter as u8],
    //     vec![AccountMeta::new(counter_keypair_pub, true)],
    // );

    let data = CounterInstruction::InitTree("Test-string2".to_string());

    let dec_instruction = Instruction::new_with_borsh(
        program_id,
        &data,
        vec![
            // AccountMeta::new(counter_keypair_pub, true),
            AccountMeta::new(counter_keypair_pub, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[dec_instruction], Some(&payer.pubkey()));
    tx.sign(signers, recent_blockhash);
    bank_clients.process_transaction(tx).await.unwrap();

    let account = match bank_clients.get_account(counter_keypair_pub).await {
        Ok(x) => x,
        Err(err) => {
            panic_log(format!("failed to get counter account: {}", err));
        }
    };

    if let Some(account_data) = account {
        let mut counter_data =
            solana_program::borsh1::try_from_slice_unchecked::<CounterAccount>(&account_data.data)
                .unwrap();

        if let CounterInstruction::InitTree(init_str) = data {
            println!("root_hash {}", counter_data.root_hash);
            println!("leafs {:?}", counter_data.leafs);

            counter_data.leafs.pop();
            counter_data.leafs.push(init_str);

            let tree = merkle_tree::MerkleTree::new(&counter_data.leafs);

            let new_hash = *tree.get_root().unwrap();
            assert_eq!(counter_data.root_hash, new_hash);
        }
    } else {
        panic_log(format!("No counter account found"));
    }
}



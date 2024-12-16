use std::{rc::Rc, time::Duration, thread::sleep};
use anchor_client::{
    solana_sdk::{
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        system_program,
        commitment_config::CommitmentConfig,
        // pubkey::Pubkey,
    },
    Client, Cluster,
};

use merkle::{accounts, MerkleAccount, HASH_SIZE, LeafInserted};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let payer = read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))?;
    let url = Cluster::Devnet;
    
    let client = Client::new_with_options(
        url,
        Rc::new(payer),
        CommitmentConfig::processed()
    );

    let program_id = "4ekk1PnQEv3ak6kv88UChc1uc7769FgMNgdgT5h5m3qB".parse()?;
    let program = client.program(program_id)?;

    let (sender, receiver) = std::sync::mpsc::channel();
    let event_listener = program.on::<LeafInserted>(move |program_id, event| {
        println!("Получено событие!");
        println!("Лист: {:?}", event.leaf);
        println!("Индекс: {}", event.index);
        println!("Новый корень: {:?}", event.root);
        if sender.send(event).is_err() {
            println!("Ошибка при отправке события через канал");
        }
    })?;

    sleep(Duration::from_secs(1));

    let merkle_account = Keypair::new();
    println!("Создание нового аккаунта дерева Меркла: {}", merkle_account.pubkey());
    
    program
        .request()
        .accounts(accounts::Initialize {
            merkle_account: merkle_account.pubkey(),
            user: program.payer(),
            system_program: system_program::ID,
        })
        .signer(&merkle_account)
        .args(merkle::instruction::Initialize)
        .send()?;

    let test_leaves = vec![
        [1u8; HASH_SIZE],
        [2u8; HASH_SIZE],  
        [3u8; HASH_SIZE], 
        [4u8; HASH_SIZE], 
        [5u8; HASH_SIZE],  
    ];

    for (i, leaf) in test_leaves.iter().enumerate() {
        println!("Добавление листа {}...", i + 1);
        program
            .request()
            .accounts(accounts::InsertLeaf {
                merkle_account: merkle_account.pubkey(),
                authority: program.payer(),
            })
            .args(merkle::instruction::InsertLeaf {
                leaf: *leaf,
            })
            .send()?;

  
        let account: MerkleAccount = program.account(merkle_account.pubkey())?;
        println!("Лист {}: добавлен", i + 1);
        println!("Текущий корень: {:?}", account.root);
        println!("Количество листьев: {}", account.leaf_count);
        println!("-------------------");
    }

    let account: MerkleAccount = program.account(merkle_account.pubkey())?;
    println!("\n");
    println!("Финальное состояние дерева:");
    println!("Корень: {:?}", account.root);
    println!("Всего листьев: {}", account.leaf_count);

    event_listener.unsubscribe();

    Ok(())
}

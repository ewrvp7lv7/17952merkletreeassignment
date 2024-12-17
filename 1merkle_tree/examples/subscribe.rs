use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::signature::{read_keypair_file, Signer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = read_keypair_file("./target/deploy/merkle_tree_program-keypair.json").unwrap();
    let program_id = keypair.pubkey().to_string();

    // Connect to the Solana devnet
    let rpc_url = String::from("ws://127.0.0.1:8900");

    let (mut sub, rx) = solana_client::pubsub_client::PubsubClient::logs_subscribe(
        rpc_url.as_str(),
        // mentions: [ <string> ] - array containing a single Pubkey (as base-58 encoded string); 
        //if present, subscribe to only transactions mentioning this address
        RpcTransactionLogsFilter::Mentions(vec![program_id]),
        RpcTransactionLogsConfig { commitment: None },
    )?;

    println!("Subscriber started...");

    while let Ok(msg) = rx.recv() {
        println!("{:?}", msg);
        // for l in msg.value.logs {
        //     println!("{}", l);
        // }
    }

    sub.shutdown().unwrap();

    Ok(())
}

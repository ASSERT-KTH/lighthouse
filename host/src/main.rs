
use std::process::exit;

use alloy_primitives::FixedBytes;
use host::{execute_proof, submit_verify_transaction};
use tokio::task;
use hex::decode;

#[tokio::main]
async fn main() {

    task::spawn(async {
        // This is a random BLS12-318 key
        let pk_hex = "47d96b38e4c7228b4ce87d123b43fa465bc69ffe64ee95903938bbc3ea764e28";
        //let pk_hex = "202432893ce01cf5a0774079606b550a846901dac27fb846b3e8a940be97df15";
        let pk_bytes = decode(pk_hex).unwrap();

        let msg = FixedBytes::from_slice(pk_bytes.as_slice());
        // signing the same private key just because I'm lazy
        let p = match execute_proof(pk_bytes.as_slice(), &msg.into()).await {
            Ok(proof) => {
                println!("Proof executed successfully");
                proof
            }
            Err(e) => {
                eprintln!("error while generating proof {}", e);
                exit(1);
            }
        };

        let tx_hash = submit_verify_transaction(p).await;
        println!("published with hash {}", tx_hash);

    })
    .await
    .expect("Task failed");


}

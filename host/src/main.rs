
use std::process::exit;

use host::{execute_proof, submit_verify_transaction};
use tokio::task;


#[tokio::main]
async fn main() {

    task::spawn(async {
        let pk_hex = "0x12345678";
        let p = match execute_proof(pk_hex).await {
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

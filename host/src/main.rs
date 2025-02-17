
use host::execute_proof;
use tokio::task;

#[tokio::main]
async fn main() {
    task::spawn(async move {
        let pk_hex = "0x12345678";
        match execute_proof(pk_hex).await {
            Ok(_) => println!("Proof executed successfully"),
            Err(e) => eprintln!("Failed to execute proof: {}", e),
        }
    })
    .await
    .expect("Task failed");
}

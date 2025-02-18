use std::str::FromStr;
use url::Url;

use methods::{HELLO_GUEST_ELF, HELLO_GUEST_ID, HELLO_GUEST_PATH};
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv, ProverOpts, VerifierContext};
use risc0_ethereum_contracts::{alloy::hex::ToHexExt, encode_seal};

use alloy::{
    network::EthereumWallet, providers::ProviderBuilder, signers::local::PrivateKeySigner
};

use alloy_primitives::Address;

alloy::sol!(
    #[sol(rpc, all_derives)]
    "./contracts/IRiscZeroVerifier.sol"
);

pub struct ProofData {
    pub seal: String,
    pub journal_digest: String,
    pub elf_id: String,
}

pub async fn submit_verify_transaction(proof_data: ProofData) -> String {
    //this is a test private key, never commit a real key to vcs
    let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2").unwrap();
    //using docker interface
    let rpc_url = Url::from_str("http://172.17.0.1:32002").unwrap();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(wallet_private_key))
        .on_http(rpc_url);

    let contract_addr = Address::parse_checksummed("0x123463a4B065722E99115D6c222f267d9cABb524", None).unwrap();

    let contract = IRiscZeroVerifier::new(contract_addr, provider);
    let image_id_bytes = alloy_primitives::FixedBytes::from_str(proof_data.elf_id.as_str()).unwrap();
    let journal_digest_bytes = alloy_primitives::FixedBytes::from_str(proof_data.journal_digest.as_str()).unwrap();
    let seal_bytes = alloy_primitives::Bytes::from_str(proof_data.seal.as_str()).unwrap();
    let call_builder = contract.verify(seal_bytes, image_id_bytes, journal_digest_bytes);

    //let handle = tokio::runtime::Handle::current();

    // let pending_tx = handle.block_on(call_builder.send()).unwrap();
    let pending_tx = call_builder.send().await.unwrap();
    pending_tx.tx_hash().encode_hex()
    //let _ = pending_tx.expect("error").get_receipt().await;
}

pub fn u32_array_to_hex_string(arr: &[u32]) -> String {
    arr.iter()
        .map(|&num| format!("{:08x}", num.to_be()))
        .collect::<Vec<String>>()
        .join("")
}

pub async fn execute_proof(pk_hex: &str) -> Result<ProofData, Box<dyn std::error::Error>> {
    // Initialize tracing.
    // tracing_subscriber::fmt()
    //     .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
    //     .init();
    // let testpk_hex = "0x1234";

    let exec_env = ExecutorEnv::builder()
    .write(&pk_hex)
    .unwrap()
    .build()
    .unwrap();



    // Obtain the default prover.
    let prover = default_prover();

    // Proof information by proving the specified ELF binary.
    // This struct contains the receipt along with statistics about execution of the guest


    println!("workspace: {:?}", std::env::current_dir().unwrap());
    println!("GUEST_ADDR: {}", HELLO_GUEST_PATH);
    println!("GUEST_ELF length: {}", HELLO_GUEST_ELF.len());
    println!("GUEST_ELF_ID length: {}", HELLO_GUEST_ID.len());
    let prove_info = prover
        .prove_with_ctx(
            exec_env,
            &VerifierContext::default(),
            HELLO_GUEST_ELF,
            &ProverOpts::groth16()
        )
        .unwrap();

    // extract the receipt.
    let receipt = prove_info.receipt;

    // For example:
    //let output: u32 = receipt.journal.decode().unwrap_or_else(|error| {
    //    eprintln!("Failed to open the file: {}", error);
    //    std::process::exit(1); // Gracefully exit the program
    //});

    let encoded = encode_seal(&receipt).unwrap().encode_hex();
    let journal = receipt.journal.bytes.clone();
    let journal_digest = journal.digest().encode_hex();

    println!("I generated a proof of execution! {} is a public output from journal ", encoded);

    let p = ProofData {
        seal: encoded,
        elf_id: u32_array_to_hex_string(&HELLO_GUEST_ID),
        journal_digest
    };

    Ok(p)
}



use methods::{HELLO_GUEST_ELF, HELLO_GUEST_ID, HELLO_GUEST_PATH};
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv, ProverOpts, VerifierContext};
use risc0_ethereum_contracts::{alloy::hex::ToHexExt, encode_seal};
use std::{env, io::Read};

pub fn u32_array_to_hex_string(arr: &[u32]) -> String {
    arr.iter()
        .map(|&num| format!("{:08x}", num.to_be()))
        .collect::<Vec<String>>()
        .join("")
}

pub async fn execute_proof(pk_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    // Decode Journal: Upon receiving the proof, the application decodes the journal to extract
    // the verified number. This ensures that the number being submitted to the blockchain matches
    // the number that was verified off-chain.
    //let y = U256::from(output);
    //let journal_digest = U256::abi_encode(&y);

    println!("journal digest as is: 0x{}", journal.digest());


    //let hash = Sha256::digest(&journal_digest);

    println!("encoded seal: 0x{}", encoded);
    println!("elf id: 0x{}", u32_array_to_hex_string(&HELLO_GUEST_ID));

    //receipt
    //    .verify(HELLO_GUEST_ID)
    //    .unwrap();

    println!("I generated a proof of execution! {} is a public output from journal ", encoded);

    Ok(())
}

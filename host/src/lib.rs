use std::str::FromStr;
use url::Url;
use tokio::fs;
use risc0_ethereum_contracts::alloy::hex::ToHexExt;
use risc0_ethereum_contracts::encode_seal;
use hzys_produce_attestation::{hzys_produce_unaggregated_attestation,Electra_enabled, CACHE_ITEM,ATTESTATION_BASE,Slot as HzysSlot,EarlyAttesterCache,CommitteeIndex,AttestationBase,HeadBeaconState,AttesterCacheKey,HEADBEACONSTATE,ATTESTER_CACHE_KEY};
use methods::{VERCLIENT1_ELF, VERCLIENT1_ID, VERCLIENT1_PATH, VERCLIENT2_ELF, VERCLIENT2_ID, VERCLIENT2_PATH, VERCLIENT3_ELF, VERCLIENT3_ID, VERCLIENT3_PATH};
use risc0_zkvm::{default_prover, sha::Digestible, ExecutorEnv, ProverOpts, VerifierContext};
use std::time::Instant;

use alloy::{
    network::EthereumWallet, providers::ProviderBuilder, signers::local::PrivateKeySigner, providers::Provider
};

use alloy_primitives::Address;
use alloy_primitives::U256;
use alloy_primitives::Bytes;
// use alloy_sol_macro::{sol};

alloy::sol!(
    #[sol(rpc, all_derives)]
    "./contracts/VerifiableClientDiversity.sol"
);

#[derive( Clone)]
pub struct ProofData {
    pub seal: String,
    pub journal_digest: String,
    pub elf_id: String,
}

pub async fn get_version_index() {

   let get_index = |id: [u32; 8]| async move {
        // 将 ID 转换为 bytes32
        let hex_str = u32_array_to_hex_string(&id);
        let version_hash = alloy_primitives::FixedBytes::from_str(&hex_str).expect("Invalid hex string");

        // 初始化钱包和提供者
        let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2")
            .expect("Invalid private key");
        let rpc_url = Url::from_str("http://172.17.0.1:32002").expect("Invalid RPC URL");
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(wallet_private_key))
            .on_http(rpc_url);

        // 合约地址
        let contract_addr = Address::parse_checksummed("0x777777A4B065722E99115D6c222f267d9CaBB524", None)
            .expect("Invalid contract address");

        // 创建合约实例
        let contract = VerifiableClientDiversity::new(contract_addr, provider.clone());

        // 调用 getVersionIndex 函数
        let return_data = contract.getVersionIndex(version_hash).call().await.expect("Call failed");

        // 提取返回值
        return_data._0
    };

    let index1 = get_index(VERCLIENT1_ID).await;
    let index2 = get_index(VERCLIENT2_ID).await;
    let index3 = get_index(VERCLIENT3_ID).await;
    println!("hzysdebuginfo: Version index for verclient1: {}, verclient2: {}, verclient3: {} ", index1, index2, index3);
}

pub async fn get_minority_version() -> String {
    let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2").unwrap();
    let rpc_url = Url::from_str("http://172.17.0.1:32002").unwrap();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(wallet_private_key))
        .on_http(rpc_url);

    let contract_addr = Address::parse_checksummed("0x777777A4B065722E99115D6c222f267d9CaBB524", None).unwrap();

    let contract = VerifiableClientDiversity::new(contract_addr, provider.clone());

    let return_data = contract.getMinority().call().await.unwrap();

    let version_hash = return_data.versionHash;

    let hex_string = version_hash.encode_hex();

    println!("hzysdebuginfo: Minority version hash: {}", hex_string);

    hex_string
}

// pub async fn sumbit_addversion_transaction(){
//     let add_version = |verclient_id: [u32; 8]| async move {
//         let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2").unwrap();
//         //using docker interface
//         let rpc_url = Url::from_str("http://172.17.0.1:32002").unwrap();

//         let provider = ProviderBuilder::new()
//             .with_recommended_fillers()
//             .wallet(EthereumWallet::from(wallet_private_key))
//             .on_http(rpc_url);

//         let contract_addr = Address::parse_checksummed("0x777777A4B065722E99115D6c222f267d9CaBB524", None).unwrap();
//         let contract = VerifiableClientDiversity::new(contract_addr, provider.clone());
//         let image_id_bytes = alloy_primitives::FixedBytes::from_str(
//             u32_array_to_hex_string(&verclient_id).as_str()
//         ).unwrap();

//         let call_builder = contract.addVersion(image_id_bytes);
//         let pending_tx: alloy::providers::PendingTransactionBuilder<alloy::transports::http::Http<alloy::transports::http::Client>, alloy::network::Ethereum>
//             = call_builder.send().await.unwrap();

//         let tx_hash =pending_tx.tx_hash().encode_hex();
//         // let r = provider.get_transaction_receipt(tx_hash.parse().unwrap()).await.unwrap();
//         // let status = r.clone().unwrap().inner.status();
//         // if status  {
//         //     println!("✅ OK");
//         // } else {
//         //     println!("❌ Failed");
//         // }

//         // if let logs = r.clone().unwrap().inner.logs() {
//         //     for log in logs {
//         //         println!("📦 {:?}", log);
//         //     }
//         // }
//     };

//     add_version(VERCLIENT1_ID).await;
//     add_version(VERCLIENT2_ID).await;
//     add_version(VERCLIENT3_ID).await;
// }

pub async fn submit_RISC0verify_transaction(slot:u64, proof_data: ProofData) -> String {
    println!("\n\n\n\n risc zero verify transaction\n\n\n\n");
    get_version_index().await;
    //this is a test private key, never commit a real key to vcs
    let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2").unwrap();
    //using docker interface
    let rpc_url = Url::from_str("http://172.17.0.1:32002").unwrap();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(wallet_private_key))
        .on_http(rpc_url);

    let contract_addr = Address::parse_checksummed("0x777777A4B065722E99115D6c222f267d9CaBB524", None).unwrap();

    let contract = VerifiableClientDiversity::new(contract_addr, provider.clone());
    let image_id_bytes = alloy_primitives::FixedBytes::from_str(proof_data.elf_id.as_str()).unwrap();
    let journal_digest_bytes = alloy_primitives::FixedBytes::from_str(proof_data.journal_digest.as_str()).unwrap();
    let seal_bytes = alloy_primitives::Bytes::from_str(proof_data.seal.as_str()).unwrap();
    let slot_u256 = U256::from(slot);
    let call_builder = contract.submitProof(slot_u256, seal_bytes, image_id_bytes, journal_digest_bytes);

    //let handle = tokio::runtime::Handle::current();

    // let pending_tx = handle.block_on(call_builder.send()).unwrap();
    let pending_tx: alloy::providers::PendingTransactionBuilder<alloy::transports::http::Http<alloy::transports::http::Client>, alloy::network::Ethereum> = call_builder.send().await.unwrap();

    let tx_hash =pending_tx.tx_hash().encode_hex();
    // let r = provider.get_transaction_receipt(tx_hash.clone().parse().unwrap()).await.unwrap();
    // let status = r.clone().unwrap().inner.status();
    // if status  {
    //     println!("✅ OK");
    // } else {
    //     println!("❌ Failed");
    // }

    // if let logs = r.clone().unwrap().inner.logs() {
    //     for log in logs {
    //         println!("📦 {:?}", log);
    //     }
    // }
    tx_hash
    //let _ = pending_tx.expect("error").get_receipt().await;
}


pub async fn submit_TEEverify_transaction(slot:u64, elf_id: String) -> String {
    //this is a test private key, never commit a real key to vcs
    let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2").unwrap();
    //using docker interface
    let rpc_url = Url::from_str("http://172.17.0.1:32002").unwrap();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(wallet_private_key))
        .on_http(rpc_url);

    let contract_addr = Address::parse_checksummed("0x12345696a9b74165be0ac84260CC14fC1C0eF5FF", None).unwrap();

    let contract =  VerifiableClientDiversity::new(contract_addr, provider);
    let image_id_bytes = alloy_primitives::FixedBytes::from_str(elf_id.as_str()).unwrap();
    let journal_digest_bytes = alloy_primitives::FixedBytes::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let filebytes: Vec<u8> = fs::read("/root/quote-1.dat")
        .await
        .expect("read quote-1.dat failed");
    let seal_bytes: Bytes = Bytes::from(filebytes);
    let slot_u256 = U256::from(slot);

    let call_builder = contract.submitProof(slot_u256, seal_bytes, image_id_bytes, journal_digest_bytes);


    //let handle = tokio::runtime::Handle::current();

    // let pending_tx = handle.block_on(call_builder.send()).unwrap();
    let pending_tx: alloy::providers::PendingTransactionBuilder<alloy::transports::http::Http<alloy::transports::http::Client>, alloy::network::Ethereum> = call_builder.send().await.unwrap();
    pending_tx.tx_hash().encode_hex()
    //let _ = pending_tx.expect("error").get_receipt().await;
}

pub fn u32_array_to_hex_string(arr: &[u32]) -> String {
    arr.iter()
        .map(|&num| format!("{:08x}", num.to_be()))
        .collect::<Vec<String>>()
        .join("")
}

pub async fn attestation_execute_proof(
    request_slot: HzysSlot,
    request_index: CommitteeIndex,
    context_early_attester_cache: EarlyAttesterCache,
    context_sepc: bool,
    context_attestation_base: AttestationBase,
    context_beacon_state: HeadBeaconState,
    context_attester_cache_key: AttesterCacheKey,
   ) -> Result<ProofData, Box<dyn std::error::Error>>
{
    let Parm_slot = request_slot.clone();

    let exec_env = ExecutorEnv::builder()
    .write(&Parm_slot)
    .unwrap()
    .write(&request_index)
    .unwrap()
    .write(&context_early_attester_cache)
    .unwrap()
    .write(&context_sepc)
    .unwrap()
    .write(&context_attestation_base)
    .unwrap()
    .write(&context_beacon_state)
    .unwrap()
    .write(&context_attester_cache_key)
    .unwrap()
    .build()
    .unwrap();



    //
    let generate_proof = move |elf: &[u8], id: [u32; 8], path: &str| -> Result<ProofData, Box<dyn std::error::Error>> {
        let prover = default_prover();
        println!("GUEST_ADDR: {}", path);
        println!("GUEST_ELF length: {}", elf.len());
        println!("GUEST_ELF_ID length: {}", id.len());

        let start_time = Instant::now();
        let prove_info = prover
            .prove_with_ctx(
                exec_env,
                &VerifierContext::default(),
                elf,
                &ProverOpts::groth16()
            )
             .unwrap();

        let elapsed_time = start_time.elapsed();
        println!("Execution time: {:.2?}", elapsed_time);

        let receipt = prove_info.receipt;

        let output: u32 = receipt.journal.decode().unwrap_or_else(|error| {
            eprintln!("Failed to decode journal: {}", error);
            std::process::exit(1);
        });

        let encoded = encode_seal(&receipt).unwrap().encode_hex();
        let journal = receipt.journal.bytes.clone();
        let journal_digest = journal.digest().encode_hex();

        println!("I generated a proof of execution! {} is a public output from journal", encoded);

        let p = ProofData {
            seal: encoded,
            elf_id: u32_array_to_hex_string(&id),
            journal_digest,
        };

        Ok(p)
    };

    let CLIENT1_ID_HEX:String = u32_array_to_hex_string(&VERCLIENT1_ID);
    let CLIENT2_ID_HEX:String = u32_array_to_hex_string(&VERCLIENT2_ID);
    let CLIENT3_ID_HEX:String = u32_array_to_hex_string(&VERCLIENT3_ID);

    let version = get_minority_version().await;
    println!("{} {} {} {} {}", CLIENT1_ID_HEX,"\n", CLIENT2_ID_HEX,"\n", CLIENT3_ID_HEX);

    let version = get_minority_version().await;
    println!("{} {} {}", CLIENT1_ID_HEX, CLIENT2_ID_HEX, CLIENT3_ID_HEX);

    match version.as_str() { // 将 String 转为 &str 以进行比较
        _ if version == CLIENT1_ID_HEX => {
            println!("hzysdebuginfo: minority version is verclient1, version: {}", CLIENT1_ID_HEX);
            let result = generate_proof(VERCLIENT1_ELF, VERCLIENT1_ID, VERCLIENT1_PATH)?;
            return Ok(result);
        }
        _ if version == CLIENT2_ID_HEX => {
            println!("hzysdebuginfo: minority version is verclient2, version: {}", CLIENT2_ID_HEX);
            let result = generate_proof(VERCLIENT2_ELF, VERCLIENT2_ID, VERCLIENT2_PATH)?;
            return Ok(result);
        }
        _ if version == CLIENT3_ID_HEX => {
            println!("hzysdebuginfo: minority version is verclient3, version: {}", CLIENT3_ID_HEX);
            let result = generate_proof(VERCLIENT3_ELF, VERCLIENT3_ID, VERCLIENT3_PATH)?;
            return Ok(result);
        }
        _ => return Err("No minority version found".into()),
    }
    // if (get_minority_version().await != u32_array_to_hex_string(&VERCLIENT1_ID)) {
    //     let result = generate_proof(VERCLIENT1_ELF, VERCLIENT1_ID, VERCLIENT1_PATH)?;
    //     Ok(result)
    // } else if (get_minority_version().await != u32_array_to_hex_string(&VERCLIENT2_ID)) {
    //     let result = generate_proof(VERCLIENT2_ELF, VERCLIENT2_ID, VERCLIENT2_PATH)?;
    //     Ok(result)
    // } else if (get_minority_version().await != u32_array_to_hex_string(&VERCLIENT3_ID)) {
    //     let result = generate_proof(VERCLIENT3_ELF, VERCLIENT3_ID, VERCLIENT3_PATH)?;
    //     Ok(result)
    // } else {
    //     Err("No minority version found".into())
    // }

    // let p1 = ProofData {
    //     seal: String::from("example_seal"),
    //     journal_digest: String::from("example_digest"),
    //     elf_id: String::from("example_elf_id"),
    // };

    // Ok(p1)

}

// pub async fn execute_proof(pk_hex: &[u8], msg: &[u8; 32]) -> Result<ProofData, Box<dyn std::error::Error>> {
//     // Initialize tracing.
//     // tracing_subscriber::fmt()
//     //     .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
//     //     .init();
//     // let testpk_hex = "0x1234";

//     let exec_env = ExecutorEnv::builder()
//     .write(&pk_hex)
//     .unwrap()
//     .write(&msg)
//     .unwrap()
//     .build()
//     .unwrap();



//     // Obtain the default prover.
//     let prover = default_prover();

//     // Proof information by proving the specified ELF binary.
//     // This struct contains the receipt along with statistics about execution of the guest


//     //println!("workspace: {:?}", std::env::current_dir().unwrap());
//     println!("GUEST_ADDR: {}", HELLO_GUEST_PATH);
//     println!("GUEST_ELF length: {}", HELLO_GUEST_ELF.len());
//     println!("GUEST_ELF_ID length: {}", HELLO_GUEST_ID.len());
//     let prove_info = prover
//         .prove_with_ctx(
//             exec_env,
//             &VerifierContext::default(),
//             HELLO_GUEST_ELF,
//             &ProverOpts::groth16()
//         )
//         .unwrap();

//     // extract the receipt.
//     let receipt = prove_info.receipt;

//     // For example:
//     //let output: u32 = receipt.journal.decode().unwrap_or_else(|error| {
//     //    eprintln!("Failed to open the file: {}", error);
//     //    std::process::exit(1); // Gracefully exit the program
//     //});

//     let encoded = encode_seal(&receipt).unwrap().encode_hex();
//     let journal = receipt.journal.bytes.clone();
//     let journal_digest = journal.digest().encode_hex();

//     println!("I generated a proof of execution! {} is a public output from journal ", encoded);

//     let p = ProofData {
//         seal: encoded,
//         elf_id: u32_array_to_hex_string(&HELLO_GUEST_ID),
//         journal_digest
//     };

//     Ok(p)
// }



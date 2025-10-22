use std::str::FromStr;
use alloy::hex;
use alloy::hex::FromHex;
use alloy::hex::ToHexExt;
use serde::Deserialize;
use url::Url;

use alloy::{
    network::EthereumWallet, providers::ProviderBuilder, signers::local::PrivateKeySigner, providers::Provider
};

use alloy_primitives::Address;
use alloy_primitives::U256;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::sync::Mutex;
use serde_json::{json, to_string};
use lazy_static::lazy_static;
use tokio::io::AsyncWriteExt;

use reqwest::blocking::Client;
use serde::Serialize;

alloy::sol!(
    #[sol(rpc, all_derives)]
    "./contracts/VerifiableClientDiversity.sol"
);

lazy_static! {
    static ref LOG_FILE: Mutex<Option<tokio::fs::File>> = Mutex::new(None);
}


#[derive( Clone)]
pub struct ProofData {
    pub seal: String,
    pub journal_digest: String,
    pub elf_id: String,
}

pub async fn get_version_index(contract_addr: String, slot:u64) {
    let client1_id = String::from_str("53fc6ec3c41446582a704e4f2c0f0685c69109aa364329407121c47c6928dbab").unwrap();
    let client2_id = String::from_str("4584a90e2c2479f5d810dd9d3ed9ff278481b9a13344daa44bd586b6061ab609").unwrap();
    let client3_id = String::from_str("213c52aefbfe129f724205a645e0b21ad3bd156f1c884eb8c952c8e3feb58999").unwrap();

    let mut file = {
        let mut lock = LOG_FILE.lock().await;
        if lock.is_none() {
            let path = Path::new("/index_logs");
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .await
                .expect("Failed to open log file");

            *lock = Some(file);
        }
        // 将文件句柄移出 Mutex
        lock.take().unwrap()
    };

   let get_index = |id: String, contract_address: String| async move {
        // 将 ID 转换为 bytes32
        let hex_str = id.clone();
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
        let contract_address_1 = Address::parse_checksummed(contract_address, None)
            .expect("Invalid contract address");

        // 创建合约实例
        let contract = VerifiableClientDiversity::new(contract_address_1, provider.clone());

        // 调用 getVersionIndex 函数
        let return_data = contract.getVersionIndex(version_hash).call().await.expect("Call failed");

        // 提取返回值
        return_data._0
    };

    let index1 = get_index(client1_id.clone(), contract_addr.clone()).await;
    let index2 = get_index(client2_id.clone(), contract_addr.clone()).await;
    let index3 = get_index(client3_id.clone(), contract_addr.clone()).await;
    println!("slot: {} verclient1: {}, verclient2: {}, verclient3: {} ",slot, index1, index2, index3);

    let data = json!({
        "slot": slot,
        "verclient1": index1,
        "verclient2": index2,
        "verclient3": index3,
    });
    let data_str = to_string(&data).expect("Failed to serialize JSON");
    if let Err(e) = file.write_all(data_str.as_bytes()).await {
        eprintln!("Failed to write to log file: {}", e);
    }
    if let Err(e) = file.write_all(b"\n").await {
        eprintln!("Failed to write newline to log file: {}", e);
    }
}

pub async fn get_minority_version(contract_addr: String) -> String {
    let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2").unwrap();
    let rpc_url = Url::from_str("http://172.17.0.1:32002").unwrap();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(wallet_private_key))
        .on_http(rpc_url);

    let contract_addr = Address::parse_checksummed(contract_addr, None).unwrap();

    let contract = VerifiableClientDiversity::new(contract_addr, provider.clone());

    let return_data = contract.getMinority().call().await.unwrap();

    let version_hash = return_data.versionHash;

    let hex_string = version_hash.encode_hex();

    println!("hzysdebuginfo: Minority version hash: {}", hex_string);

    hex_string
}

pub async fn submit_RISC0verify_transaction(contract_addr: String, slot:u64, proof_data: ProofData) -> String {
    println!("\n\n\n\n risc zero verify transaction\n\n\n\n");
    get_version_index(contract_addr.clone(), slot).await;
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


pub async fn submit_TEEverify_transaction(slot:u64, proof: ProofData) -> String {
    let contract_addr = "0x12345696a9b74165be0ac84260CC14fC1C0eF5FF".to_string();
    //this is a test private key, never commit a real key to vcs
    get_version_index(contract_addr.clone(), slot).await;
    let wallet_private_key = PrivateKeySigner::from_str("a291e47eca2999e09be704728c686c854bdc69e972b1f229d2bfb532ec23f3e2").unwrap();
    //using docker interface
    let rpc_url = Url::from_str("http://172.17.0.1:32002").unwrap();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(wallet_private_key))
        .on_http(rpc_url);

    let contract_addr = Address::parse_checksummed(contract_addr.clone(), None).unwrap();

    let contract =  VerifiableClientDiversity::new(contract_addr, provider);
    let image_id_bytes = alloy_primitives::FixedBytes::from_hex(proof.elf_id.as_str()).unwrap();
    let journal_digest_bytes = alloy_primitives::FixedBytes::from_hex(proof.journal_digest.as_str()).unwrap();
    let seal_bytes = alloy_primitives::Bytes::from_hex(proof.seal.as_str()).unwrap();
    println!("attestation: {}", proof.seal);
    println!("image_id: {}", proof.elf_id);
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


#[derive(Serialize)]
pub struct AttestRequest {
    pub idx: String,
    pub sk: String,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttestResponse {
    pub signature: String,
    pub attestation: String,
}

pub async fn execute_in_tee(pk_hex: &[u8], msg: &[u8; 32]) -> Result<ProofData, Box<dyn std::error::Error>> {

    let generate_proof = move |id: String| -> Result<ProofData, Box<dyn std::error::Error>> {

        let client = Client::new();

        let body = AttestRequest{
            idx: id.clone(),
            sk: hex::encode(pk_hex),
            msg: hex::encode(msg),
        };

        let res = client
            .post("http://172.211.132.32:3000/attest")
            .json(&body)
            .send()
            .unwrap();


        let attestation: AttestResponse = res.json().unwrap();


        println!("I generated an attestation!");

        let p = ProofData {
            seal: attestation.attestation,
            elf_id: id.clone(),
            journal_digest: id.clone(),
        };

        Ok(p)
    };

    let contract_addr = "0x12345696a9b74165be0ac84260CC14fC1C0eF5FF";

    let version = get_minority_version(contract_addr.to_string()).await;

    let result = generate_proof(version)?;

    Ok(result)

}


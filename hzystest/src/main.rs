
use std::process::exit;

use alloy_primitives::FixedBytes;
use tokio::task;
use alloy_primitives::hex::decode;
use state_processing::per_epoch_processing::{process_epoch, EpochProcessingSummary};
use beacon_chain::test_utils::BeaconChainHarness;
use beacon_chain::types::{EthSpec, MinimalEthSpec,MainnetEthSpec};
// use bls::{FixedBytesExtended, Hash256};
// use types::{attestation, Slot};
// use hzys_produce_attestation::{hzys_produce_unaggregated_attestation,Electra_enabled, CACHE_ITEM,ATTESTATION_BASE,Slot as HzysSlot,EarlyAttesterCache,HEADBEACONSTATE,ATTESTER_CACHE_KEY,AttesterCacheKey,HeadBeaconState};


#[tokio::main(worker_threads = 6)]
pub async fn main() {
    // 定义要并发执行的任务数量
    let task_count = 10;

    // 创建一个向量来存储任务句柄
    let mut handles = Vec::with_capacity(task_count);

    // 启动多个异步任务
    for i in 0..task_count {
        let handle = tokio::spawn(async move {
            println!("Task {i} is running...");
            generate_attestation().await;
            println!("Task {i} completed.");
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Task failed: {:?}", e);
        }
    }
}

pub async fn generate_attestation() {
    // Produce 2 epochs, or 64 blocks
    let num_blocks_produced = MainnetEthSpec::slots_per_epoch() * 2;
    let validators_keypairs =
    types::test_utils::generate_deterministic_keypairs(100);

    let harness = BeaconChainHarness::builder(MainnetEthSpec)
        .default_spec()
        .keypairs(validators_keypairs)
        .fresh_ephemeral_store()
        .mock_execution_layer()
        .build();
    harness.advance_slot();

    let chain = &harness.chain;
    let current_slot = chain.slot().expect("should get slot");
    let mut valid_attestation = chain
    .produce_unaggregated_attestation(current_slot, 0)
    .expect("should not error while producing attestation");


    print!("slot:{}\nIndex:{}\nsource:{}\ntarget:{}\n ", valid_attestation.data().slot, valid_attestation.data().index,valid_attestation.data().source.epoch, valid_attestation.data().target.epoch);

    // attestation_execute_proof
    // (
    //     HzysSlot::new(current_slot.value()),
    //     0,
    // ).await;

}

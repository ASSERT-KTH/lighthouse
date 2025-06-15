
use std::process::exit;

use alloy_primitives::FixedBytes;
use host::attestation_execute_proof;
use tokio::task;
use alloy_primitives::hex::decode;
use state_processing::per_epoch_processing::{process_epoch, EpochProcessingSummary};
use tokio::fs;
use alloy_primitives::Bytes;
use alloy_primitives::hex::encode;
// use beacon_chain::test_utils::BeaconChainHarness;
// use beacon_chain::types::{EthSpec, MinimalEthSpec,MainnetEthSpec};
// use bls::{FixedBytesExtended, Hash256};
// use types::{attestation, Slot};
// use hzys_produce_attestation::{hzys_produce_unaggregated_attestation,Electra_enabled, CACHE_ITEM,ATTESTATION_BASE,Slot as HzysSlot,EarlyAttesterCache,HEADBEACONSTATE,ATTESTER_CACHE_KEY,AttesterCacheKey,HeadBeaconState};


#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt()
    // .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
    // .init();

    task::spawn(async {
        let proof_type: u32 = 1;
        let filebytes: Vec<u8> = fs::read("/mnt/nvme/zheyuan/lighthouse/quote-1.dat")
        .await
        .expect("read quote-1.dat failed");
        let seal_bytes: Bytes = Bytes::from(filebytes);
        println!("seal_bytes: 0x{}", encode(seal_bytes));

        // if proof_type == 1 {
        //     // generate_attestation().await;
        // } else {
        //     let pk_hex = "47d96b38e4c7228b4ce87d123b43fa465bc69ffe64ee95903938bbc3ea764e28";
        //     //let pk_hex = "202432893ce01cf5a0774079606b550a846901dac27fb846b3e8a940be97df15";
        //     let pk_bytes = decode(pk_hex).unwrap();

        //     // let msg = FixedBytes::from_slice(pk_bytes.as_slice());
        //     // signing the same private key just because I'm lazy
        //     // let p = match execute_proof(pk_bytes.as_slice(), &msg.into()).await {
        //     //     Ok(proof) => {
        //     //         println!("Proof executed successfully");
        //     //         proof
        //     //     }
        //     //     Err(e) => {
        //     //         eprintln!("error while generating proof {}", e);
        //     //         exit(1);
        //     //     }
        //     // };
        //     // let tx_hash = submit_verify_transaction(p).await;
        //     // println!("published with hash {}", tx_hash);
        // }

    })
    .await
    .expect("Task failed");


}

// pub async fn generate_attestation() {
//     // Produce 2 epochs, or 64 blocks
//     let num_blocks_produced = MainnetEthSpec::slots_per_epoch() * 2;
//     let validators_keypairs =
//     types::test_utils::generate_deterministic_keypairs(100);

//     let harness = BeaconChainHarness::builder(MainnetEthSpec)
//         .default_spec()
//         .keypairs(validators_keypairs)
//         .fresh_ephemeral_store()
//         .mock_execution_layer()
//         .build();
//     harness.advance_slot();

//     let chain = &harness.chain;
//     let current_slot = chain.slot().expect("should get slot");
//     let mut valid_attestation = chain
//     .produce_unaggregated_attestation(current_slot, 0)
//     .expect("should not error while producing attestation");


//     print!("slot:{}\nIndex:{}\nsource:{}\ntarget:{}\n ", valid_attestation.data().slot, valid_attestation.data().index,valid_attestation.data().source.epoch, valid_attestation.data().target.epoch);

//     attestation_execute_proof
//     (
//         HzysSlot::new(current_slot.value()),
//         0,
//     ).await;

// }

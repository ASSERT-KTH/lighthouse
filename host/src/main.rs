
use std::process::exit;
use std::str::FromStr;

use alloy_primitives::{fixed_bytes, FixedBytes};
use host::{execute_in_tee, submit_TEEverify_transaction, AttestRequest, AttestResponse};
use tokio::task;
use alloy_primitives::hex::decode;
use alloy_primitives::Bytes;
use alloy_primitives::hex::encode;
// use beacon_chain::test_utils::BeaconChainHarness;
// use beacon_chain::types::{EthSpec, MinimalEthSpec,MainnetEthSpec};
// use bls::{FixedBytesExtended, Hash256};
// use types::{attestation, Slot};
// use hzys_produce_attestation::{hzys_produce_unaggregated_attestation,Electra_enabled, CACHE_ITEM,ATTESTATION_BASE,Slot as HzysSlot,EarlyAttesterCache,HEADBEACONSTATE,ATTESTER_CACHE_KEY,AttesterCacheKey,HeadBeaconState};

fn main() {
    // tracing_subscriber::fmt()
    // .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
    // .init();

    let client = reqwest::blocking::Client::new();

        let body = AttestRequest{
            idx: "53fc6ec3c41446582a704e4f2c0f0685c69109aa364329407121c47c6928dbab".to_string(),
            sk: "16a1ec185cf2c157111868bf129c2c525393606dcaef5892e83177f8c6ae3f0d".to_string(),
            msg: "9e8ebd111fd348e0cc2d9104e52cf535c7468e15a6745e9a619d047b6d01edfc".to_string(),
        };

        let res = client
            .post("http://172.211.132.32:3000/attest")
            .json(&body)
            .send()
            .unwrap();


        let attestation: AttestResponse = res.json().unwrap();
             // signing the same private key just because I'm lazy

        println!("{:?}", attestation.attestation);
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

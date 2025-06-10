// use blst::min_pk::SecretKey;
use risc0_zkvm::guest::env;
use hzys_produce_attestation::{hzys_produce_unaggregated_attestation,Electra_enabled, CACHE_ITEM,ATTESTATION_BASE,Slot as HzysSlot,EarlyAttesterCache,AttestationBase,HeadBeaconState,AttesterCacheKey};

fn main() {
    let start = env::cycle_count();
    let mut Hzys_slot:HzysSlot=env::read();
    let mut Hzys_requestindex:u64=env::read();
    let mut Hzys_early_attester_cache:EarlyAttesterCache=env::read();

    let mut Hzys_spec_flag:bool=env::read();
    let mut Hys_attestation:AttestationBase=env::read();
    let mut Hys_beaconstate:HeadBeaconState=env::read();
    let mut Hys_attester_cache_key:AttesterCacheKey=env::read();

    let v1: u64 = 3;
    let mut tmp: u64 = 0;
    if v1== 1 {
        tmp=v1;
    }

    hzys_produce_unaggregated_attestation(Hzys_slot,
    Hzys_requestindex,
    &Hzys_early_attester_cache,
    Hzys_spec_flag,
    &Hys_attestation,
    &Hys_beaconstate,
    &Hys_attester_cache_key
    );

    let end = env::cycle_count();
    env::commit(&Hzys_spec_flag);
    eprintln!("my_operation_to_measure: {}", end - start);
}


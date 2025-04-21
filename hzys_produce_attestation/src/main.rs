use hzys_produce_attestation::{hzys_produce_unaggregated_attestation,Slot, CommitteeIndex};


fn main() {
    let slot = Slot(10);
    let committee_index: CommitteeIndex = 5; // 使用 5 作为示例值
    // let result = hzys_produce_unaggregated_attestation(slot,committee_index);
    // match result {
    //     Ok(_) => println!("Attestation produced successfully."),
    //     Err(e) => eprintln!("Error producing attestation: {}", e),
    // }
}

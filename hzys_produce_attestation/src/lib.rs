use fixed_bytes;
use safe_arith::{SafeArith};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Deserialize, Serialize)]
pub struct Slot(pub u64);

impl Slot {
    pub fn value(&self) -> u64 {
        self.0
    }

    pub const fn new(slot: u64) -> Slot {
        Slot(slot)
    }

    pub fn epoch(self, slots_per_epoch: u64) -> Epoch {
        Epoch::new(self.0)
            .safe_div(slots_per_epoch)
    }

    pub fn saturating_sub(&self, other_value:u64) -> Slot {
        Slot(self.0.saturating_sub(other_value))
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Copy)]
pub struct Epoch(pub u64);

impl Epoch {

    pub fn value(&self) -> u64 {
        self.0
    }

    pub const fn new(epoch: u64) -> Epoch {
        Epoch(epoch)
    }

    pub fn safe_div(&self, rhs: u64) -> Self {
        if rhs == 0 {
            panic!("Division by zero");
        }
        match self.0.checked_div(rhs) {
            Some(result) => Epoch::new(result),
            None => panic!("Arithmetic overflow"),
        }
    }

    pub fn start_slot(&self, slots_per_epoch: u64) -> Slot {
        Slot::new(self.0.saturating_mul(slots_per_epoch))
    }

}

pub type CommitteeIndex = u64;
//pub type Hash256 = fixed_bytes::Hash256;
pub type Hash256 = [u8; 32];


#[derive(Debug, PartialEq, Clone, Deserialize, Serialize,Copy)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: Hash256,
}

impl Checkpoint {
    pub fn new(epoch: Epoch, root: Hash256) -> Self {
        Checkpoint { epoch, root }
    }
}

#[derive(Debug, PartialEq, Clone,Deserialize, Serialize,Copy)]
pub struct CommitteeLengths {
    /// The `epoch` to which the lengths pertain.
    epoch: Epoch,
    /// The length of the shuffling in `self.epoch`.
    active_validator_indices_len: usize,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct EarlyAttesterCache {
    pub item: Option<CacheItem>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize,Copy)]
pub struct CacheItem {
    epoch: Epoch,
    beacon_block_root: Hash256,
    source: Checkpoint,
    target: Checkpoint,
    block_slot: Slot,
    committee_count:usize,
    committee_len:usize
}

impl EarlyAttesterCache {

    pub fn get_epoch_or_default(&self) -> Epoch {
        self.item
            .as_ref()
            .map(|item| item.epoch.clone())
            .unwrap_or(Epoch(0)) // 默认值为 Epoch(0)
    }

    pub fn get_block_slot_or_default(&self) -> Slot {
        self.item
            .as_ref()
            .map(|item| item.block_slot.clone(),)
            .unwrap_or(Slot(0)) // 默认值为 Slot(0)
    }

    pub fn get_committee_count_or_default(&self) -> usize {
        self.item
            .as_ref()
            .map(|item| item.committee_count)
            .unwrap_or(0) // 默认值为 0
    }

    pub fn get_committee_len_or_default(&self) -> usize {
        self.item
            .as_ref()
            .map(|item| item.committee_len)
            .unwrap_or(0) // 默认值为 0
    }
}


thread_local! {
    pub static CACHE_ITEM: RefCell<CacheItem> = RefCell::new(CacheItem {
        epoch: Epoch(0),
        beacon_block_root: [0; 32],
        source: Checkpoint {
            epoch: Epoch(0),
            root: [0; 32],
        },
        target: Checkpoint {
            epoch: Epoch(0),
            root: [0; 32],
        },
        block_slot: Slot(0),
        committee_count: 0,
        committee_len: 0,
    });
}

thread_local! {
    pub static Electra_enabled: RefCell<bool> = RefCell::new(false);
}


pub fn set_epoch(epoch: u64) {
    CACHE_ITEM.with(|item| {
        let mut item = item.borrow_mut();
        item.epoch = Epoch(epoch);
    });
}

// 设置 beacon_block_root
// pub fn set_beacon_block_root(root: Hash256) {
//     CACHE_ITEM.with(|item| {
//         let mut item = item.borrow_mut();
//         item.beacon_block_root = root;
//     });
// }

// 设置 source
pub fn set_source(source_epoch: u64, source_root: Hash256) {
    CACHE_ITEM.with(|item| {
        let mut item = item.borrow_mut();
        item.source = Checkpoint {
            epoch: Epoch(source_epoch),
            root: source_root,
        };
    });
}

// 设置 target
pub fn set_target(target_epoch: u64, target_root: Hash256) {
    CACHE_ITEM.with(|item| {
        let mut item = item.borrow_mut();
        item.target = Checkpoint {
            epoch: Epoch(target_epoch),
            root: target_root,
        };
    });
}

// 设置 block_slot
pub fn set_block_slot(slot: u64) {
    CACHE_ITEM.with(|item| {
        let mut item = item.borrow_mut();
        item.block_slot = Slot(slot);
    });
}

// 设置 committee_count
pub fn set_committee_count(count: usize) {
    CACHE_ITEM.with(|item| {
        let mut item = item.borrow_mut();
        item.committee_count = count;
    });
}

pub fn set_committee_len(len: usize) {
    CACHE_ITEM.with(|item| {
        let mut item = item.borrow_mut();
        item.committee_len = len;
    });
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize,Copy)]
pub struct AttestationData {
    pub slot: Slot,
    pub index: u64,

    // LMD GHOST vote
    pub beacon_block_root: Hash256,

    // FFG Vote
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct AttestationBase {
    pub eelectra_committee_bits: Vec<bool>,
    pub aggregation_bits: Vec<bool>,
    pub data: AttestationData,
    pub signature: Vec<u8>,
}

// global variable
thread_local! {
    pub static ATTESTATION_BASE: RefCell<AttestationBase> = RefCell::new(AttestationBase {
        eelectra_committee_bits: vec![],
        aggregation_bits: vec![],
        data: AttestationData {
            slot: Slot(0),
            index: 0,
            beacon_block_root: [0; 32],
            source: Checkpoint {
                epoch: Epoch(0),
                root: [0; 32],
            },
            target: Checkpoint {
                epoch: Epoch(0),
                root: [0; 32],
            },
        },
        signature: vec![],
    });
}


    // 设置 data.slot
    pub fn set_slot(slot: Slot) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().data.slot = slot;
        });
    }

    // 设置 data.index
    pub fn set_index(index: u64) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().data.index = index;
        });
    }

    // 设置 data.beacon_block_root
    pub fn set_beacon_block_root(root: [u8; 32]) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().data.beacon_block_root = root;
        });
    }

    // 设置 data.source.epoch
    pub fn set_source_epoch(epoch: Epoch) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().data.source.epoch = epoch;
        });
    }

    // 设置 data.source.root
    pub fn set_source_root(root: [u8; 32]) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().data.source.root = root;
        });
    }

    // 设置 data.target.epoch
    pub fn set_target_epoch(epoch: Epoch) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().data.target.epoch = epoch;
        });
    }

    // 设置 data.target.root
    pub fn set_target_root(root: [u8; 32]) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().data.target.root = root;
        });
    }

    // 设置 signature
    pub fn set_signature(signature: Vec<u8>) {
        ATTESTATION_BASE.with(|base| {
            base.borrow_mut().signature = signature;
        });
    }

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct HeadBeaconState {
    pub finalized_slot: Slot,
    pub headslot: Slot,
    pub beacon_block_root: Hash256,
    pub beacon_state_root: Hash256,
    pub target_block_root: Hash256,
    pub head_epoch: Epoch,
    pub head_current_epoch_attesting_info: Option<(Checkpoint, usize)>,
}

impl HeadBeaconState {
    // 获取 finalized_slot 的方法
    pub fn get_finalized_slot(&self) -> Slot {
        self.finalized_slot
    }

    pub fn get_headslot(&self) -> Slot {
        self.headslot
    }

    pub fn get_beacon_block_root(&self) -> Hash256 {
        self.beacon_block_root
    }

    pub fn get_beacon_state_root(&self) -> Hash256 {
        self.beacon_state_root
    }

    pub fn get_target_block_root(&self) -> Hash256 {
        self.target_block_root
    }

    pub fn get_head_epoch(&self) -> Epoch {
        self.head_epoch
    }

    pub fn get_head_current_epoch_attesting_info(&self) -> Option<(Checkpoint, usize)> {
        self.head_current_epoch_attesting_info
    }
}

// global variable
thread_local! {
    pub static HEADBEACONSTATE: RefCell<HeadBeaconState> = RefCell::new(HeadBeaconState {
        finalized_slot: Slot(0),
        headslot: Slot(0),
        beacon_block_root: [0; 32],
        beacon_state_root: [0; 32],
        target_block_root: [0; 32],
        head_epoch: Epoch(0),
        head_current_epoch_attesting_info:Some((Checkpoint {
            epoch: Epoch(0),
            root: [0; 32],
        }, 42))
    });
}

pub fn set_finalized_slot(slot: Slot) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().finalized_slot = slot;
    });
}

pub fn set_headslot(slot: Slot) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().headslot = slot;
    });
}

pub fn set_beacon_block_root_head(root: [u8; 32]) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().beacon_block_root = root;
    });
}

pub fn set_beacon_state_root(root: [u8; 32]) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().beacon_state_root = root;
    });
}

pub fn set_target_block_root(root: [u8; 32]) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().target_block_root = root;
    });
}

pub fn set_head_epoch(epoch: Epoch) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().head_epoch = epoch;
    });
}

pub fn set_head_current_epoch_attesting_info(info: Option<(Checkpoint, usize)>) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().head_current_epoch_attesting_info = info;
    });
}




#[derive(Debug)]
pub enum Error {
    AttestingToFinalizedSlot {
        finalized_slot: Slot,
        request_slot: Slot,
    },
    AttestingToAncientSlot {
        lowest_permissible_slot: Slot,
        request_slot: Slot,
    },
}

pub fn hzys_produce_unaggregated_attestation(
    request_slot: Slot,
    request_index: CommitteeIndex,
    context_early_attester_cache: &EarlyAttesterCache,
    context_sepc: bool,
    context_attestation_base: &AttestationBase,
    context_beacon_state: &HeadBeaconState,
)-> Result<Option<AttestationBase>, Error> {

    match  try_attest(request_slot, request_index, context_early_attester_cache, context_sepc,context_attestation_base){
        Ok(Some(attestation)) => return Ok(Some(attestation)),
        Ok(None) => return Ok(None),
        Err(e) => {
            println!("Error: {:?}", e);
            return Err(e);
        }
    }
    let slots_per_epoch = 32;
    let request_epoch = request_slot.epoch(slots_per_epoch);


    let beacon_block_root;
    let beacon_state_root;
    let target;
    let current_epoch_attesting_info: Option<(Checkpoint, usize)>;

    // let slots_per_epoch = 32;
    // let request_epoch = request_slot.epoch(slots_per_epoch);

    let finalized_slot = context_beacon_state.get_finalized_slot();

    if request_slot < finalized_slot {
        return Err(Error::AttestingToFinalizedSlot {
            request_slot: request_slot,
            finalized_slot: finalized_slot,
        });
    }

    let slots_per_historical_root: u64 = 8192;
    let headslot= context_beacon_state.get_headslot();
    let lowest_permissible_slot =
        headslot.saturating_sub(slots_per_historical_root);
    if request_slot < lowest_permissible_slot {
        return Err(Error::AttestingToAncientSlot {
            lowest_permissible_slot,
            request_slot,
        });
    }

    if request_slot >= headslot {
        beacon_block_root= context_beacon_state.get_beacon_block_root();
        beacon_state_root= context_beacon_state.get_beacon_state_root();
    }else{
        beacon_block_root= context_beacon_state.get_beacon_block_root();
        beacon_state_root= context_beacon_state.get_beacon_state_root();
    }

    let target_slot = request_epoch.start_slot(slots_per_epoch);
    let target_root= if headslot<=target_slot{
        beacon_block_root
    }else{
        context_beacon_state.get_target_block_root()
    };

    target = Checkpoint {
        epoch: request_epoch,
        root: target_root,
    };

    current_epoch_attesting_info = if context_beacon_state.get_head_epoch()==request_epoch{
        context_beacon_state.get_head_current_epoch_attesting_info()
    }else{
        None
    };


    Ok(None)

}


pub fn try_attest(
    request_slot: Slot,
    request_index: CommitteeIndex,
    context_early_attester_cache: &EarlyAttesterCache,
    context_sepc: bool,
    context_attestation_base: &AttestationBase,
    // context_attestation_base: &AttestationBase,
) -> Result<Option<AttestationBase>, Error>  {
    /*omit rwlocker*/
    let request_epoch = request_slot.epoch(32);

    if request_epoch != context_early_attester_cache.get_epoch_or_default() {
        return Ok(None);
    }

    if request_slot < context_early_attester_cache.get_block_slot_or_default() {
        return Ok(None);
    }

    let committee_count = context_early_attester_cache.get_committee_count_or_default();
    if request_index >= committee_count as u64 {
        return Ok(None);
    }

    let committee_len = context_early_attester_cache.get_committee_len_or_default();

    Ok(Some(context_attestation_base.clone()))
    // Ok(Some(context_attestation_base.clone()))
 }



// pub fn print_cache_item() {
//     CACHE_ITEM.with(|cache| {
//         if let Some(cache_item) = cache.borrow().as_ref() {
//             println!("{}", cache_item); // 使用 Display trait 输出
//         } else {
//             println!("CacheItem is not initialized");
//         }
//     });
// }

use fixed_bytes;
use safe_arith::{SafeArith};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;



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

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Copy, Hash, Eq)]
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


#[derive(Debug, PartialEq, Clone, Deserialize, Serialize,Copy, Hash, Eq)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: Hash256,
}

impl Checkpoint {
    pub fn new(epoch: Epoch, root: Hash256) -> Self {
        Checkpoint { epoch, root }
    }
}

#[derive(Debug, PartialEq, Clone,Deserialize, Serialize,Copy, Hash, Eq)]
pub struct CommitteeLengths {
    /// The `epoch` to which the lengths pertain.
    pub epoch: Epoch,
    /// The length of the shuffling in `self.epoch`.
    pub active_validator_indices_len: usize,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeadBeaconState {
    pub finalized_slot: Slot,
    pub headslot: Slot,
    pub beacon_block_root: Hash256,
    pub beacon_state_root: Hash256,
    pub target_block_root: Hash256,
    pub head_epoch: Epoch,
    pub head_current_epoch_attesting_info: Option<(Checkpoint, usize)>,
    pub block_execution_status:Option<ExecutionStatus>,
    pub cachevalue: Option<(Checkpoint, usize)>,
    // pub attester_cache: AttesterCache,
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

    pub fn get_block_execution_status(&self) -> Option<ExecutionStatus> {
        self.block_execution_status
    }

    pub fn get_cachevalue(&self) -> Option<(Checkpoint, usize)> {
        self.cachevalue
    }

    // pub fn get_attester_cache(&self) -> &AttesterCache {
    //     &self.attester_cache
    // }
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
        }, 42)),
        block_execution_status: None,
        cachevalue: Some((Checkpoint {
            epoch: Epoch(0),
            root: [0; 32],
        }, 42)),
        // attester_cache: AttesterCache {
        //     cache: HashMap::new(),
        // },
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

pub fn set_block_execution_status(status: Option<ExecutionStatus>) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().block_execution_status = status;
    });
}

pub fn set_cachevalue(value: Option<(Checkpoint, usize)>) {
    HEADBEACONSTATE.with(|base| {
        base.borrow_mut().cachevalue = value;
    });
}


// pub fn set_attester_cache(cache: AttesterCache) {
//     HEADBEACONSTATE.with(|base| {
//         base.borrow_mut().attester_cache = cache;
//     });
// }

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize, Hash, Eq)]
pub struct AttesterCacheKey {
    /// The epoch from which the justified checkpoint should be observed.
    ///
    /// Attestations which use `self.epoch` as `target.epoch` should use this key.
    pub epoch: Epoch,
    /// The root of the block at the last slot of `self.epoch - 1`.
    pub decision_root: Hash256,
}

// global variable
thread_local! {
    pub static ATTESTER_CACHE_KEY: RefCell<AttesterCacheKey> = RefCell::new(AttesterCacheKey {
        epoch: Epoch(0),
        decision_root: [0; 32],
    });
}

pub fn set_attester_cache_key(epoch: Epoch, decision_root: Hash256) {
    ATTESTER_CACHE_KEY.with(|key| {
        key.borrow_mut().epoch = epoch;
        key.borrow_mut().decision_root = decision_root;
    });
}

impl AttesterCacheKey {
    pub fn get_attester_cache_key(&self) -> Self {
        ATTESTER_CACHE_KEY.with(|key| {
            key.borrow().clone()
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutionBlockHash(pub Hash256);



#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// An EL has determined that the payload is valid.
    Valid(ExecutionBlockHash),
    /// An EL has determined that the payload is invalid.
    Invalid(ExecutionBlockHash),
    /// An EL has not yet verified the execution payload.
    Optimistic(ExecutionBlockHash),
    /// The block is either prior to the merge fork, or after the merge fork but before the terminal
    /// PoW block has been found.
    ///
    /// # Note:
    ///
    /// This `bool` only exists to satisfy our SSZ implementation which requires all variants
    /// to have a value. It can be set to anything.
    Irrelevant(bool),
    NoneError,
}

impl ExecutionStatus {
    pub fn is_valid_or_irrelevant(&self) -> bool {
        matches!(
            self,
            ExecutionStatus::Valid(_) | ExecutionStatus::Irrelevant(_)
        )
    }
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
    HeadBlockNotFullyVerified {
        beacon_block_root: Hash256,
        execution_status: ExecutionStatus,
    },
    HeadMissingFromForkChoice(Hash256),
    CacheValueNotFound,
}

#[derive(Debug,PartialEq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct AttesterCacheValue {
    pub current_justified_checkpoint: Checkpoint,
    pub committee_lengths: CommitteeLengths,
}


type CacheHashMap = HashMap<AttesterCacheKey, AttesterCacheValue>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttesterCache {
    cache: CacheHashMap,
}

impl AttesterCache {
    pub fn new(param_cache: CacheHashMap) -> Self {
        AttesterCache {
            cache: param_cache,
        }
    }
}

#[inline(never)]
pub fn hzys_produce_unaggregated_attestation(
    request_slot: Slot,
    request_index: CommitteeIndex,
    context_early_attester_cache: &EarlyAttesterCache,
    context_sepc: bool,
    context_attestation_base: &AttestationBase,
    context_beacon_state: &HeadBeaconState,
    context_attester_cache_key: &AttesterCacheKey,
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
    let attester_cache_key;
    {

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

        attester_cache_key = context_attester_cache_key.get_attester_cache_key();
    }
        let block_execution_status = context_beacon_state.get_block_execution_status();

        match block_execution_status
        {
            Some(execution_status) if execution_status.is_valid_or_irrelevant() => (),
            Some(execution_status) => {
                return Err(Error::HeadBlockNotFullyVerified {
                    beacon_block_root,
                    execution_status:execution_status,
                })
            }
            None => return Err(Error::HeadMissingFromForkChoice(beacon_block_root)),
        };

        let (justified_checkpoint, committee_len) =
        if let Some((justified_checkpoint, committee_len)) = current_epoch_attesting_info {
            // The head state is in the same epoch as the attestation, so there is no more
            // required information.
            (justified_checkpoint, committee_len)
        }else {
            context_beacon_state.get_cachevalue().ok_or(Error::CacheValueNotFound)?
        };



        Ok(Some(context_attestation_base.clone()))

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

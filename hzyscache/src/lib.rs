use hzys_produce_attestation::{Slot,CommitteeIndex,EarlyAttesterCache,AttestationBase,HeadBeaconState,AttesterCacheKey,CacheItem};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use sha2::{Digest, Sha256};
use bincode::{serialize, Options};
use lazy_static::lazy_static;

// 线程安全的全局缓存
lazy_static! {
    static ref CALL_HASH_CACHE: Arc<Mutex<HashSet<[u8; 32]>>> = Arc::new(Mutex::new(HashSet::new()));
}

#[derive(serde::Serialize)]
struct FunctionCallParams {
    request_slot: Slot,
    request_index: CommitteeIndex,
    context_early_attester_cache_item: Option<CacheItem>,
    context_sepc: bool,
    context_attestation_base:  AttestationBase,
    context_beacon_state:  HeadBeaconState,
    context_attester_cache_key:  AttesterCacheKey,
}

pub fn generate_call_hash(
    request_slot: Slot,
    request_index: CommitteeIndex,
    context_early_attester_cache: EarlyAttesterCache,
    context_sepc: bool,
    context_attestation_base: AttestationBase,
    context_beacon_state: HeadBeaconState,
    context_attester_cache_key: AttesterCacheKey,
) -> [u8; 32] {
    let params = FunctionCallParams {
        request_slot,
        request_index,
        context_early_attester_cache_item: context_early_attester_cache.item,
        context_sepc,
        context_attestation_base,
        context_beacon_state,
        context_attester_cache_key,
    };

    // 创建配置对象并设置 fixint 编码
    let config = bincode::options().with_fixint_encoding();

    // 使用配置对象的 serialize 方法
    let serialized = config.serialize(&params).expect("Serialization failed");

    let mut hasher = Sha256::new();
    hasher.update(serialized);
    let result = hasher.finalize();

    result.into()
}

pub fn check_duplicate_call(
    request_slot: Slot,
    request_index: CommitteeIndex,
    context_early_attester_cache: EarlyAttesterCache,
    context_sepc: bool,
    context_attestation_base: AttestationBase,
    context_beacon_state: HeadBeaconState,
    context_attester_cache_key: AttesterCacheKey,
) -> bool {
    let hash = generate_call_hash(
        request_slot,
        request_index,
        context_early_attester_cache,
        context_sepc,
        context_attestation_base,
        context_beacon_state,
        context_attester_cache_key,
    );

    let mut cache = CALL_HASH_CACHE.lock().unwrap();
    if cache.contains(&hash) {
        println!("hzysdebuginfo: hit cache, duplicate call detected");
        true // 已存在，表示重复调用
    } else {
        println!("hzysdebuginfo: miss cache, inserting new hash");
        cache.insert(hash);
        false // 未重复
    }
}

// ✅ 新增函数：清空缓存
pub fn clear_call_hash_cache() {
    let mut cache = CALL_HASH_CACHE.lock().expect("Failed to acquire lock on call hash cache");
    cache.clear();
}

// // 示例调用逻辑
// fn safe_hzys_produce_unaggregated_attestation(
//     request_slot: Slot,
//     request_index: CommitteeIndex,
//     context_early_attester_cache: &EarlyAttesterCache,
//     context_sepc: bool,
//     context_attestation_base: &AttestationBase,
//     context_beacon_state: &HeadBeaconState,
//     context_attester_cache_key: &AttesterCacheKey,
// ) -> Result<Option<AttestationBase>, Error> {
//     if !check_duplicate_call(
//         request_slot,
//         request_index,
//         context_early_attester_cache,
//         context_sepc,
//         context_attestation_base,
//         context_beacon_state,
//         context_attester_cache_key,
//     ) {
//         hzys_produce_unaggregated_attestation(
//             request_slot,
//             request_index,
//             context_early_attester_cache,
//             context_sepc,
//             context_attestation_base,
//             context_beacon_state,
//             context_attester_cache_key,
//         )
//     } else {
//         Ok(None) // 如果检测到重复调用，则直接返回 None 或者根据需求处理
//     }
// }

// // 假设这是您要包装的原函数
// fn hzys_produce_unaggregated_attestation(
//     _request_slot: Slot,
//     _request_index: CommitteeIndex,
//     _context_early_attester_cache: &EarlyAttesterCache,
//     _context_sepc: bool,
//     _context_attestation_base: &AttestationBase,
//     _context_beacon_state: &HeadBeaconState,
//     _context_attester_cache_key: &AttesterCacheKey,
// ) -> Result<Option<AttestationBase>, Error> {
//     // 实际业务逻辑
//     todo!()
// }

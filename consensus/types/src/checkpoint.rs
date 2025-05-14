use crate::test_utils::TestRandom;
use crate::{Epoch, Hash256};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::TreeHash;
use hzys_produce_attestation;

/// Casper FFG checkpoint, used in attestations.
///
/// Spec v0.12.1
#[derive(
    arbitrary::Arbitrary,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    TestRandom,
)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: Hash256,
}

impl Checkpoint {
    /// Returns a new `Checkpoint` with the given epoch and root.
    pub fn convert_checkpoint(&self) -> hzys_produce_attestation::Checkpoint {
        hzys_produce_attestation::Checkpoint {
            epoch: hzys_produce_attestation::Epoch(self.epoch.value()),
            root: self.root.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_and_tree_hash_tests!(Checkpoint);
}

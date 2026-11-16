use codec::{Decode, Encode};
use polkadot_sdk::sp_core::offchain::Duration;
pub use polkadot_sdk::sp_io::offchain::{
    local_storage_clear, local_storage_get, local_storage_set, timestamp,
};
use polkadot_sdk::sp_runtime::offchain::StorageKind;
use polkadot_sdk::sp_std::prelude::Vec;

use crate::ClusterId;

pub const IN_FLIGHT_PREFIX: &[u8] = b"ocwqf";
pub const QUARANTINE_PREFIX: &[u8] = b"ocwqq";
pub const UNREACHABLE_PREFIX: &[u8] = b"ocwqu";

/// Consecutive discovery walks in which no collector answered before a cluster is quarantined.
pub const UNREACHABLE_WALKS_BEFORE_QUARANTINE: u32 = 3;

/// Why a cluster was not admitted into the current off-chain worker pass.
#[derive(Debug, PartialEq, Eq)]
pub enum SkipReason {
    /// A previous pass died while this cluster was in flight. The quarantine was set just now.
    CrashDetected { until_millis: u64 },
    /// A quarantine set by an earlier pass is still in effect.
    Quarantined { until_millis: u64 },
}

impl SkipReason {
    pub fn until_millis(&self) -> u64 {
        match self {
            Self::CrashDetected { until_millis } | Self::Quarantined { until_millis } => {
                *until_millis
            }
        }
    }
}

/// Admission control for the per-cluster loop of an off-chain worker.
///
/// A cluster whose processing kills the worker — a wasm trap, an OOM kill, a node restart — would
/// otherwise be re-entered on every pass and starve every cluster behind it in the loop. The queue
/// marks a cluster as in flight before it is processed and clears the mark afterwards, so a mark
/// still standing on the next pass is evidence that the previous pass died inside that cluster.
/// Such a cluster is quarantined for a fixed duration and skipped, letting the rest of the loop run
/// at full rate; once the quarantine lapses it is retried once, and rejoins normally if it
/// succeeds.
///
/// Both marks live in persistent off-chain local storage and are scoped by the worker's `id`, so
/// workers in different pallets keep independent queues.
pub struct OcwClusterQueue {
    id: Vec<u8>,
    quarantine: Duration,
}

impl OcwClusterQueue {
    pub fn new(id: Vec<u8>, quarantine: Duration) -> Self {
        Self { id, quarantine }
    }

    /// Admits `cluster_id` into the current pass, or explains why it is being skipped.
    ///
    /// The returned [`ClusterPass`] clears the in-flight mark when dropped. Dropping it is what
    /// distinguishes a pass that ended — successfully or with an error — from one that never
    /// returned at all.
    pub fn try_enter(&self, cluster_id: &ClusterId) -> Result<ClusterPass, SkipReason> {
        let now = timestamp().unix_millis();
        let quarantine_key = self.quarantine_key(cluster_id);
        let in_flight_key = self.in_flight_key(cluster_id);

        if let Some(raw) = local_storage_get(StorageKind::PERSISTENT, &quarantine_key) {
            if let Ok(until_millis) = u64::decode(&mut &raw[..]) {
                if now < until_millis {
                    return Err(SkipReason::Quarantined { until_millis });
                }
            }
            // Lapsed, or written by an incompatible version. Either way it no longer applies.
            local_storage_clear(StorageKind::PERSISTENT, &quarantine_key);
        }

        if local_storage_get(StorageKind::PERSISTENT, &in_flight_key).is_some() {
            let until_millis = now.saturating_add(self.quarantine.millis());
            local_storage_set(
                StorageKind::PERSISTENT,
                &quarantine_key,
                &until_millis.encode(),
            );
            local_storage_clear(StorageKind::PERSISTENT, &in_flight_key);

            return Err(SkipReason::CrashDetected { until_millis });
        }

        local_storage_set(StorageKind::PERSISTENT, &in_flight_key, &now.encode());

        Ok(ClusterPass { key: in_flight_key })
    }

    /// Records whether a discovery walk over `cluster_id`'s collectors found one that answered.
    ///
    /// After [`UNREACHABLE_WALKS_BEFORE_QUARANTINE`] consecutive walks that found none, the
    /// cluster is quarantined for `quarantine`, and `try_enter` skips it until that lapses. Any
    /// walk that found one resets the count. The count is kept through the quarantine, so the
    /// single retry after it lapses re-arms it on failure. Returns the quarantine deadline when
    /// one was set.
    pub fn record_discovery(
        &self,
        cluster_id: &ClusterId,
        reachable: bool,
        quarantine: Duration,
    ) -> Option<u64> {
        let unreachable_key = self.unreachable_key(cluster_id);
        if reachable {
            local_storage_clear(StorageKind::PERSISTENT, &unreachable_key);
            return None;
        }

        let walks = local_storage_get(StorageKind::PERSISTENT, &unreachable_key)
            .and_then(|raw| u32::decode(&mut &raw[..]).ok())
            .unwrap_or(0)
            .saturating_add(1);
        local_storage_set(StorageKind::PERSISTENT, &unreachable_key, &walks.encode());
        if walks < UNREACHABLE_WALKS_BEFORE_QUARANTINE {
            return None;
        }

        let until_millis = timestamp()
            .unix_millis()
            .saturating_add(quarantine.millis());
        local_storage_set(
            StorageKind::PERSISTENT,
            &self.quarantine_key(cluster_id),
            &until_millis.encode(),
        );
        Some(until_millis)
    }

    pub fn in_flight_key(&self, cluster_id: &ClusterId) -> Vec<u8> {
        (IN_FLIGHT_PREFIX, &self.id, cluster_id).encode()
    }

    pub fn quarantine_key(&self, cluster_id: &ClusterId) -> Vec<u8> {
        (QUARANTINE_PREFIX, &self.id, cluster_id).encode()
    }

    pub fn unreachable_key(&self, cluster_id: &ClusterId) -> Vec<u8> {
        (UNREACHABLE_PREFIX, &self.id, cluster_id).encode()
    }
}

/// Holds the in-flight mark for one cluster for as long as that cluster is being processed.
#[must_use = "dropping the pass is what clears the in-flight mark"]
pub struct ClusterPass {
    key: Vec<u8>,
}

impl Drop for ClusterPass {
    fn drop(&mut self) {
        local_storage_clear(StorageKind::PERSISTENT, &self.key);
    }
}

#[cfg(test)]
mod tests {
    use polkadot_sdk::sp_core::{
        offchain::{testing::TestOffchainExt, OffchainDbExt, OffchainWorkerExt, Timestamp},
        H160,
    };
    use polkadot_sdk::sp_io::TestExternalities;

    use super::*;

    const QUARANTINE_MILLIS: u64 = 60_000;

    fn cluster(byte: u8) -> ClusterId {
        H160::repeat_byte(byte)
    }

    fn queue() -> OcwClusterQueue {
        OcwClusterQueue::new(
            b"test_lock".to_vec(),
            Duration::from_millis(QUARANTINE_MILLIS),
        )
    }

    fn with_offchain(test: impl FnOnce(&mut dyn FnMut(u64))) {
        let (offchain, state) = TestOffchainExt::new();
        let mut ext = TestExternalities::default();
        ext.register_extension(OffchainDbExt::new(offchain.clone()));
        ext.register_extension(OffchainWorkerExt::new(offchain));

        ext.execute_with(|| {
            let mut set_now = |millis: u64| {
                state.write().timestamp = Timestamp::from_unix_millis(millis);
            };
            test(&mut set_now);
        });
    }

    #[test]
    fn admits_a_cluster_seen_for_the_first_time() {
        with_offchain(|_set_now| {
            assert!(queue().try_enter(&cluster(1)).is_ok());
        });
    }

    #[test]
    fn readmits_a_cluster_whose_previous_pass_returned() {
        with_offchain(|_set_now| {
            let queue = queue();
            drop(queue.try_enter(&cluster(1)).expect("first pass admitted"));

            assert!(queue.try_enter(&cluster(1)).is_ok());
        });
    }

    #[test]
    fn quarantines_a_cluster_whose_previous_pass_never_returned() {
        with_offchain(|_set_now| {
            let queue = queue();
            // A wasm trap or an OOM kill leaves the mark standing; nothing runs `Drop`.
            core::mem::forget(queue.try_enter(&cluster(1)).expect("first pass admitted"));

            assert_eq!(
                queue.try_enter(&cluster(1)).err(),
                Some(SkipReason::CrashDetected {
                    until_millis: QUARANTINE_MILLIS
                })
            );
        });
    }

    #[test]
    fn keeps_skipping_a_quarantined_cluster_until_the_quarantine_lapses() {
        with_offchain(|set_now| {
            let queue = queue();
            core::mem::forget(queue.try_enter(&cluster(1)).expect("first pass admitted"));
            let _ = queue.try_enter(&cluster(1));

            set_now(QUARANTINE_MILLIS - 1);
            assert_eq!(
                queue.try_enter(&cluster(1)).err(),
                Some(SkipReason::Quarantined {
                    until_millis: QUARANTINE_MILLIS
                })
            );

            set_now(QUARANTINE_MILLIS);
            assert!(queue.try_enter(&cluster(1)).is_ok());
        });
    }

    #[test]
    fn requarantines_a_cluster_that_dies_again_on_retry() {
        with_offchain(|set_now| {
            let queue = queue();
            core::mem::forget(queue.try_enter(&cluster(1)).expect("first pass admitted"));
            let _ = queue.try_enter(&cluster(1));

            set_now(QUARANTINE_MILLIS);
            core::mem::forget(queue.try_enter(&cluster(1)).expect("retry admitted"));

            assert_eq!(
                queue.try_enter(&cluster(1)).err(),
                Some(SkipReason::CrashDetected {
                    until_millis: QUARANTINE_MILLIS * 2
                })
            );
        });
    }

    #[test]
    fn quarantines_only_the_cluster_that_died() {
        with_offchain(|_set_now| {
            let queue = queue();
            core::mem::forget(queue.try_enter(&cluster(1)).expect("first pass admitted"));
            let _ = queue.try_enter(&cluster(1));

            for other in 2..=10u8 {
                assert!(
                    queue.try_enter(&cluster(other)).is_ok(),
                    "cluster {other} was skipped"
                );
            }
        });
    }

    #[test]
    fn keeps_queues_of_different_workers_independent() {
        with_offchain(|_set_now| {
            let inspection = queue();
            let payout = OcwClusterQueue::new(
                b"other_lock".to_vec(),
                Duration::from_millis(QUARANTINE_MILLIS),
            );

            core::mem::forget(
                inspection
                    .try_enter(&cluster(1))
                    .expect("first pass admitted"),
            );
            let _ = inspection.try_enter(&cluster(1));

            assert!(payout.try_enter(&cluster(1)).is_ok());
        });
    }

    const UNREACHABLE_MILLIS: u64 = 10_000;

    fn record_unreachable(queue: &OcwClusterQueue, byte: u8) -> Option<u64> {
        queue.record_discovery(
            &cluster(byte),
            false,
            Duration::from_millis(UNREACHABLE_MILLIS),
        )
    }

    #[test]
    fn quarantines_a_cluster_after_consecutive_walks_find_no_collector() {
        with_offchain(|_set_now| {
            let queue = queue();
            for _ in 1..UNREACHABLE_WALKS_BEFORE_QUARANTINE {
                assert_eq!(record_unreachable(&queue, 1), None);
                assert!(queue.try_enter(&cluster(1)).is_ok());
            }

            assert_eq!(record_unreachable(&queue, 1), Some(UNREACHABLE_MILLIS));
            assert_eq!(
                queue.try_enter(&cluster(1)).err(),
                Some(SkipReason::Quarantined {
                    until_millis: UNREACHABLE_MILLIS
                })
            );
        });
    }

    #[test]
    fn a_walk_that_finds_a_collector_resets_the_count() {
        with_offchain(|_set_now| {
            let queue = queue();
            for _ in 1..UNREACHABLE_WALKS_BEFORE_QUARANTINE {
                record_unreachable(&queue, 1);
            }
            queue.record_discovery(&cluster(1), true, Duration::from_millis(UNREACHABLE_MILLIS));

            for _ in 1..UNREACHABLE_WALKS_BEFORE_QUARANTINE {
                assert_eq!(record_unreachable(&queue, 1), None);
            }
            assert!(queue.try_enter(&cluster(1)).is_ok());
        });
    }

    #[test]
    fn retries_an_unreachable_cluster_once_its_quarantine_lapses_and_rearms_on_failure() {
        with_offchain(|set_now| {
            let queue = queue();
            for _ in 0..UNREACHABLE_WALKS_BEFORE_QUARANTINE {
                record_unreachable(&queue, 1);
            }

            set_now(UNREACHABLE_MILLIS);
            drop(
                queue
                    .try_enter(&cluster(1))
                    .expect("retried after the quarantine lapsed"),
            );
            assert_eq!(record_unreachable(&queue, 1), Some(UNREACHABLE_MILLIS * 2));
            assert!(queue.try_enter(&cluster(1)).is_err());
        });
    }

    #[test]
    fn an_unreachable_quarantine_lapses_after_its_own_duration_not_the_crash_one() {
        with_offchain(|set_now| {
            let queue = queue();
            for _ in 0..UNREACHABLE_WALKS_BEFORE_QUARANTINE {
                record_unreachable(&queue, 1);
            }

            set_now(UNREACHABLE_MILLIS);
            assert!(UNREACHABLE_MILLIS < QUARANTINE_MILLIS);
            assert!(queue.try_enter(&cluster(1)).is_ok());
        });
    }

    #[test]
    fn an_unreachable_cluster_does_not_hold_back_others_or_another_worker() {
        with_offchain(|_set_now| {
            let payout = queue();
            let inspection = OcwClusterQueue::new(
                b"other_lock".to_vec(),
                Duration::from_millis(QUARANTINE_MILLIS),
            );
            for _ in 0..UNREACHABLE_WALKS_BEFORE_QUARANTINE {
                record_unreachable(&payout, 1);
            }

            assert!(payout.try_enter(&cluster(1)).is_err());
            assert!(payout.try_enter(&cluster(2)).is_ok());
            assert!(inspection.try_enter(&cluster(1)).is_ok());
        });
    }
}

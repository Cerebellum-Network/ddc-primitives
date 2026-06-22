use polkadot_sdk::sp_runtime::DispatchError;

use crate::BucketId;
#[cfg(feature = "runtime-benchmarks")]
use crate::{BucketParams, ClusterId};
pub trait BucketManager<T: polkadot_sdk::frame_system::Config> {
    fn get_bucket_owner_id(bucket_id: BucketId) -> Result<T::AccountId, DispatchError>;

    #[cfg(feature = "runtime-benchmarks")]
    fn create_bucket(
        cluster_id: &ClusterId,
        bucket_id: BucketId,
        owner_id: T::AccountId,
        bucket_params: BucketParams,
    ) -> Result<(), DispatchError>;
}

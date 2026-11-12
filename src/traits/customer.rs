use polkadot_sdk::sp_runtime::DispatchError;

use crate::BucketId;

pub trait CustomerVisitor<T: polkadot_sdk::frame_system::Config> {
	fn get_bucket_owner(bucket_id: &BucketId) -> Result<T::AccountId, DispatchError>;
}

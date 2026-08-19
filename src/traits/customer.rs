use polkadot_sdk::sp_runtime::DispatchError;

use crate::{BucketId, ClusterId};

// todo(yahortsaryk): should be refactored to charge a batch of customers in one call
pub trait CustomerCharger<T: polkadot_sdk::frame_system::Config> {
    fn charge_customer(
        customer: T::AccountId,
        payout_vault: T::AccountId,
        cluster_id: ClusterId,
        amount: u128,
    ) -> Result<u128, DispatchError>;
}

pub trait CustomerDepositor<T: polkadot_sdk::frame_system::Config> {
    fn deposit(
        customer: T::AccountId,
        cluster_id: ClusterId,
        amount: u128,
    ) -> Result<(), DispatchError>;

    fn deposit_extra(
        customer: T::AccountId,
        cluster_id: ClusterId,
        amount: u128,
    ) -> Result<(), DispatchError>;

    fn deposit_for(
        funder: T::AccountId,
        customer: T::AccountId,
        cluster_id: ClusterId,
        amount: u128,
    ) -> Result<(), DispatchError>;
}

pub trait CustomerVisitor<T: polkadot_sdk::frame_system::Config> {
    fn get_bucket_owner(bucket_id: &BucketId) -> Result<T::AccountId, DispatchError>;
}

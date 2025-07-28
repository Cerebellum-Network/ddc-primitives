#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use crate::contracts::{ClusterId, AccountId, Balance};

#[ink::trait_definition]
pub trait DdcPayoutsPayer {
    #[ink(message)]
    fn charge(
        &mut self,
        vault: AccountId,
        batch: Vec<(AccountId, Balance)>,
    ) -> Vec<(AccountId, Balance)>;
}

#[ink::event]
pub struct DdcBalanceDeposited {
    #[ink(topic)]
    pub cluster_id: ClusterId,
    #[ink(topic)]
    pub owner_id: AccountId,
    pub amount: Balance,
}

#[ink::event]
pub struct DdcBalanceUnlocked {
    #[ink(topic)]
    pub cluster_id: ClusterId,
    #[ink(topic)]
    pub owner_id: AccountId,
    pub amount: Balance,
}

#[ink::event]
pub struct DdcBalanceWithdrawn {
    #[ink(topic)]
    pub cluster_id: ClusterId,
    #[ink(topic)]
    pub owner_id: AccountId,
    pub amount: Balance,
}

#[ink::event]
pub struct DdcBalanceCharged {
    #[ink(topic)]
    pub cluster_id: ClusterId,
    #[ink(topic)]
    pub owner_id: AccountId,
    pub charged: Balance,
    pub expected: Balance,
}

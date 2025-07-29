#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use crate::contracts::{ClusterId, AccountId, Balance};
use crate::BlockNumber;


pub mod traits {
    use super::*;
    use super::types::Ledger;
    use super::errors::Error;

    /// This trait is required to be implemented by any customer deposit contract 
    /// as it enables fetching of customer balances in DDC cluster required by the protocol.
    #[ink::trait_definition]
    pub trait DdcBalancesFetcher {
        /// Fetches customer balance in DDC cluster.
        #[ink(message)]
        fn get_balance(&self, owner: AccountId) -> Option<Ledger>;
    }

    /// This trait is optional to be implemented by any customer deposit contract 
    /// as it enables utilities (i.e. wallet, payment gateway, ramp service, etc.) that are not required by the protocol.
    #[ink::trait_definition]
    pub trait DdcBalancesDepositor {
        /// Top up deposit balance on behalf its owner.
        #[ink(message, payable)]
        fn deposit(&mut self) -> Result<(), Error>;

        /// Top up deposit balance for specific owner on behalf faucet.
        #[ink(message, payable)]
		fn deposit_for(&mut self, owner: AccountId) -> Result<(), Error>;

        /// Initiate unlocking of deposit balance on behalf its owner.
        #[ink(message)]
		fn unlock_deposit(&mut self, value: Balance) -> Result<(), Error>;

        /// Withdraw unlocked deposit balance on behalf its owner.
        #[ink(message)]
		fn withdraw_unlocked(&mut self) -> Result<(), Error>;
    }

    /// This trait is required to be implemented by any customer deposit contract 
    /// as it enables charges for DDC service required by the protocol.
    #[ink::trait_definition]
    pub trait DdcPayoutsPayer {
        /// Charges customers for DDC service usage while DAC-based payouts are in progress.
        #[ink(message)]
        fn charge(
            &mut self,
            vault: AccountId,
            batch: Vec<(AccountId, Balance)>,
        ) -> Vec<(AccountId, Balance)>;
    }
}


pub mod events {
    use super::*;

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
}


pub mod types {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Ledger {
        /// The owner account whose balance is actually locked and can be used to pay for DDC
        /// network usage.
        pub owner: AccountId,
        /// The total amount of the owner's balance that we are currently accounting for.
        /// It's just `active` plus all the `unlocking` balances.
        pub total: Balance,
        /// The total amount of the owner's balance that will be accessible for DDC network payouts
        /// in any forthcoming rounds.
        pub active: Balance,
        /// Any balance that is becoming free, which may eventually be transferred out of the owner
        /// (assuming that the content owner has to pay for network usage). It is assumed that this
        /// will be treated as a first in, first out queue where the new (higher value) eras get
        /// pushed on the back.
        pub unlocking: Vec<UnlockChunk>,
    }
    
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct UnlockChunk {
        /// Amount of funds to be unlocked.
        pub value: Balance,
        /// Block number at which point it'll be unlocked.
        pub block: BlockNumber,
    }
}

pub mod errors {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        Code(u16),
    }
}
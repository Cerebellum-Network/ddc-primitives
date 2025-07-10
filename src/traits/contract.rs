#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use sp_runtime::AccountId32;

#[ink::trait_definition]
pub trait DdcPayoutsPayer {
	#[ink(message)]
	fn charge(
		&mut self,
		vault: AccountId32,
		batch: Vec<(AccountId32, u128)>,
	) -> Vec<(AccountId32, u128)>;
}

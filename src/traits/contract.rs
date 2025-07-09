#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use crate::AccountId;

#[ink::trait_definition]
pub trait DdcPayoutsPayer {
	#[ink(message)]
	fn charge(
		&mut self,
		vault: AccountId,
		batch: Vec<(AccountId, u128)>,
	) -> Result<Vec<(AccountId, u128)>, ()>;
}

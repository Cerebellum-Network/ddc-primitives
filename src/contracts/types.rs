use polkadot_sdk::sp_runtime::AccountId32;

/// Cluster ID type used accross contracts extensions
pub type ClusterId = [u8; 20];

/// Account ID type used accross contracts extensions
pub type AccountId = AccountId32;

/// Balance type used accross contracts extensions
pub type Balance = u128;

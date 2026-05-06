#![cfg_attr(not(feature = "std"), no_std)]

use blake2::{Blake2s256, Digest};
use codec::{Decode, Encode};
use frame_support::parameter_types;
use polkadot_ckb_merkle_mountain_range::Merge;
use scale_info::{
    prelude::{format, string::String, vec::Vec},
    TypeInfo,
};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::KeyTypeId, hash::H160, H256, DecodeWithMemTracking};
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    AccountId32, MultiSignature, OpaqueExtrinsic, Perquintill, RuntimeDebug,
};
use sp_std::collections::btree_set::BTreeSet;
pub mod traits;
pub mod contracts;
use sp_std::str::FromStr;
pub mod ocw_mutex;

parameter_types! {
    pub MaxHostLen: u8 = 255;
    pub MaxDomainLen: u8 = 255;
}

/// An index to a block.
pub type BlockNumber = u32;
/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;
/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;
/// Balance of an account.
pub type Balance = u128;
/// Type used for expressing timestamp.
pub type Moment = u64;
/// Index of a transaction in the chain.
pub type Nonce = u32;
/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Timestamp = u64;
/// Digest item type.
pub type DigestItem = generic::DigestItem;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;
pub const MAX_PAYOUT_BATCH_COUNT: u16 = 1000;
pub const MAX_PAYOUT_BATCH_SIZE: u16 = 500;
pub const MILLICENTS: u128 = 100_000;
pub const CENTS: u128 = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: u128 = 100 * CENTS;
pub type ClusterId = H160;
pub type PaymentEra = u32;
pub type EhdEra = u32;
pub type TcaEra = u32;
pub type BucketId = u64;
pub type ClusterNodesCount = u16;
pub type StorageNodePubKey = AccountId32;
/// Hash of verified or unverified delta usage of a bucket or a node.
pub type DeltaUsageHash = H256;
/// Hash of usage that a customer is supposed to be charged for, or a provider supposed to be
/// rewarded for. Includes the current usage and verified delta usage.
pub type PayableUsageHash = H256;
/// Selective hash of sensitive information for payouts.
pub type Fingerprint = H256;

pub type BatchIndex = u16;
pub const AVG_SECONDS_MONTH: i64 = 2630016; // 30.44 * 24.0 * 3600.0;

pub struct MergeMMRHash;
impl Merge for MergeMMRHash {
    type Item = H256;
    fn merge(
        lhs: &Self::Item, // Left side of tree
        rhs: &Self::Item, // Right side of tree
    ) -> Result<Self::Item, polkadot_ckb_merkle_mountain_range::Error> {
        let mut hasher = Blake2s256::new();

        hasher.update(lhs.0.as_slice());
        hasher.update(rhs.0.as_slice());
        let hash = hasher.finalize();

        Ok(H256::from_slice(hash.as_slice()))
    }
}

// ClusterParams includes Governance non-sensetive parameters only
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterParams<AccountId> {
    pub node_provider_auth_contract: Option<AccountId>,
    pub erasure_coding_required: u32,
    pub erasure_coding_total: u32,
    pub replication_total: u32,
    pub inspection_dry_run_params: Option<InspectionDryRunParams>,
}

#[cfg(feature = "std")]
impl<AccountId> Default for ClusterParams<AccountId> {
    fn default() -> Self {
        ClusterParams {
            node_provider_auth_contract: None,
            erasure_coding_required: 0,
            erasure_coding_total: 0,
            replication_total: 0,
            inspection_dry_run_params: None,
        }
    }
}

// ClusterProtocolParams includes Governance sensitive parameters
#[derive(
    Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Default, Serialize, Deserialize,
)]
#[scale_info(skip_type_params(Balance, BlockNumber, T))]
pub struct ClusterProtocolParams<Balance, BlockNumber, AccountId> {
    pub treasury_share: Perquintill,
    pub validators_share: Perquintill,
    pub cluster_reserve_share: Perquintill,
    pub storage_bond_size: Balance,
    pub storage_chill_delay: BlockNumber,
    pub storage_unbonding_delay: BlockNumber,
    pub cost_per_mb_stored: u128,
    pub cost_per_mb_streamed: u128,
    pub cost_per_put_request: u128,
    pub cost_per_get_request: u128,
    pub cost_per_gpu_unit: u128,
    pub cost_per_cpu_unit: u128,
    pub cost_per_ram_unit: u128,
    pub customer_deposit_contract: AccountId,
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterPricingParams {
    pub cost_per_mb_stored: u128,
    pub cost_per_mb_streamed: u128,
    pub cost_per_put_request: u128,
    pub cost_per_get_request: u128,
    pub cost_per_gpu_unit: u128,
    pub cost_per_cpu_unit: u128,
    pub cost_per_ram_unit: u128,
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterFeesParams {
    pub treasury_share: Perquintill,
    pub validators_share: Perquintill,
    pub cluster_reserve_share: Perquintill,
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterBondingParams<BlockNumber> {
    pub storage_bond_size: u128,
    pub storage_chill_delay: BlockNumber,
    pub storage_unbonding_delay: BlockNumber,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo,
)]
pub struct AggregatorInfo {
    pub node_pub_key: NodePubKey,
    pub node_params: StorageNodeParams,
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub struct InspectionDryRunParams {
	pub enabled: bool,
    pub sync_node_key: NodePubKey,
    pub sync_node_params: StorageNodeParams,
}

// The `StoragePubKey` is the only variant of DDC node key. This enum should be replaced with
// trait-bounded type.
#[derive(
    Debug, Serialize, DecodeWithMemTracking, Deserialize, Clone, Ord, PartialOrd, PartialEq, Eq, Encode, Decode, TypeInfo,
)]
pub enum NodePubKey {
    StoragePubKey(StorageNodePubKey),
}

impl From<NodePubKey> for String {
    fn from(node_key: NodePubKey) -> Self {
        match node_key {
            NodePubKey::StoragePubKey(pub_key) => format!("0x{}", hex::encode(pub_key)),
        }
    }
}

impl TryFrom<String> for NodePubKey {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.starts_with("0x") || value.len() != 66 {
            return Err("NodePubKey must be a 32-byte hex string started with '0x'");
        }

        let hex_str = &value[2..]; // skip '0x'
        let hex_bytes = match hex::decode(hex_str) {
            Ok(bytes) => bytes,
            Err(_) => return Err("NodePubKey must be a valid hex string"),
        };
        if hex_bytes.len() != 32 {
            return Err("NodePubKey must be 32 chars in length");
        }
        let mut pub_key = [0u8; 32];
        pub_key.copy_from_slice(&hex_bytes[..32]);

        Ok(NodePubKey::StoragePubKey(AccountId32::from(pub_key)))
    }
}

impl TryFrom<&[u8]> for NodePubKey {
    type Error = &'static str;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err("NodePubKey must be exactly 32 bytes");
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        Ok(NodePubKey::StoragePubKey(AccountId32::new(arr)))
    }
}

impl TryFrom<Vec<u8>> for NodePubKey {
    type Error = &'static str;
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl From<NodePubKey> for Vec<u8> {
    fn from(node_key: NodePubKey) -> Self {
        match node_key {
            NodePubKey::StoragePubKey(account_id) => {
                let bytes: &[u8] = account_id.as_ref();
                bytes.to_vec()
            }
        }
    }
}

impl AsRef<[u8]> for NodePubKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            NodePubKey::StoragePubKey(account_id) => account_id.as_ref(),
        }
    }
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq)]
pub enum NodeType {
    Storage = 1,
}

impl From<NodeType> for u8 {
    fn from(node_type: NodeType) -> Self {
        match node_type {
            NodeType::Storage => 1,
        }
    }
}

impl TryFrom<u8> for NodeType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NodeType::Storage),
            _ => Err(()),
        }
    }
}

/// The type for keeping account id in hexadecimal notation (prefixed with '0x')
#[derive(
    Debug, Serialize, Deserialize, Hash, Clone, Ord, PartialOrd, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking,
)]
pub struct AccountId32Hex {
    pub id: [u8; 32],
}

impl TryFrom<String> for AccountId32Hex {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.starts_with("0x") || value.len() != 66 {
            return Err("NodePubKey must be a 32-byte hex string started with '0x'");
        }

        let hex_str = &value[2..]; // skip '0x'
        let hex_bytes = match hex::decode(hex_str) {
            Ok(bytes) => bytes,
            Err(_) => return Err("NodePubKey must be a valid hex string"),
        };
        if hex_bytes.len() != 32 {
            return Err("NodePubKey must be 32 chars in length");
        }
        let mut acc_id = [0u8; 32];
        acc_id.copy_from_slice(&hex_bytes[..32]);

        Ok(AccountId32Hex { id: acc_id })
    }
}

impl From<AccountId32Hex> for String {
    fn from(value: AccountId32Hex) -> Self {
        format!("0x{}", hex::encode(value.id.encode()))
    }
}

impl TryFrom<&str> for AccountId32Hex {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        AccountId32Hex::try_from(String::from(value))
    }
}

impl From<AccountId32Hex> for AccountId32 {
    fn from(val: AccountId32Hex) -> Self {
        AccountId32::from(val.id)
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    DecodeWithMemTracking,
    Hash,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Encode,
    Decode,
    TypeInfo,
)]
pub enum StorageNodeMode {
    /// DDC Storage node operates with enabled caching in RAM and stores data in Hard Drive
    Full = 1,
    /// DDC Storage node operates with disabled caching in RAM and stores data in Hard Drive
    Storage = 2,
    /// DDC Storage node operates with enabled caching in RAM and doesn't store data in Hard Drive
    Cache = 3,
    /// Compute Node operates with CPU, GPU and RAM resources only
    Compute = 4,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    DecodeWithMemTracking,
    Hash,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Encode,
    Decode,
    TypeInfo,
)]
pub struct StorageNodeParams {
    pub mode: StorageNodeMode,
    pub host: Vec<u8>,
    pub domain: Vec<u8>,
    pub ssl: bool,
    pub http_port: u16,
    pub grpc_port: u16,
    pub p2p_port: u16,
}

#[cfg(feature = "std")]
impl Default for StorageNodeParams {
    fn default() -> Self {
        StorageNodeParams {
            mode: StorageNodeMode::Full,
            host: Default::default(),
            domain: Default::default(),
            ssl: Default::default(),
            http_port: Default::default(),
            grpc_port: Default::default(),
            p2p_port: Default::default(),
        }
    }
}

// Params fields are always coming from extrinsic input
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq)]
pub enum NodeParams {
    StorageParams(StorageNodeParams),
}

/// DDC cluster status
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum ClusterStatus {
    Unbonded,
    Bonded,
    Activated,
    Unbonding,
}

/// DDC node kind added to DDC cluster
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum ClusterNodeKind {
    Genesis,
    External,
}

/// DDC node status in to DDC cluster
#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Serialize, Deserialize)]
pub enum ClusterNodeStatus {
    AwaitsValidation,
    ValidationSucceeded,
    ValidationFailed,
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq)]
pub struct ClusterNodeState<BlockNumber> {
    pub kind: ClusterNodeKind,
    pub status: ClusterNodeStatus,
    pub added_at: BlockNumber,
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Default)]
pub struct ClusterNodesStats {
    pub await_validation: ClusterNodesCount,
    pub validation_succeeded: ClusterNodesCount,
    pub validation_failed: ClusterNodesCount,
}

/// Stores charge in tokens(units) of customer
#[derive(PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, Default, Clone)]
pub struct CustomerCharge {
    pub transfer: u128, // charge in tokens for transferred bytes
    pub storage: u128,  // charge in tokens for stored bytes
    pub puts: u128,     // charge in tokens for number of puts
    pub gets: u128,     // charge in tokens for number of gets
    pub compute: u128   // charge in tokens for compute
}

/// Stores reward in tokens(units) of node provider
#[derive(PartialEq, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, Default, Clone)]
pub struct ProviderReward {
    pub transfer: u128, // reward in tokens for transferred bytes
    pub storage: u128,  // reward in tokens for stored bytes
    pub puts: u128,     // reward in tokens for number of puts
    pub gets: u128,     // reward in tokens for number of gets
    pub compute: u128   // reward in tokens for compute
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Default)]
pub struct MMRProof {
    pub proof: Vec<DeltaUsageHash>,
}

#[derive(Debug, PartialEq)]
pub enum NodeRepositoryError {
    StorageNodeAlreadyExists,
    StorageNodeDoesNotExist,
}

#[derive(Debug, PartialEq)]
pub enum PayoutError {
    PayoutReceiptDoesNotExist,
    InvalidPayoutReceiptParams,
}

#[derive(Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo, PartialEq, Default)]
// don't remove or change numbers, if needed add a new state to the end with new number
// DAC uses the state value for integration!
pub enum PayoutState {
    #[default]
    NotInitialized = 1,
    Initialized = 2,
    ChargingCustomers = 3,
    CustomersChargedWithFees = 4,
    RewardingProviders = 5,
    ProvidersRewarded = 6,
    Finalized = 7,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, DecodeWithMemTracking, RuntimeDebug, TypeInfo)]
pub struct BucketParams {
    pub is_public: bool,
}

pub const DAC_VERIFICATION_KEY_TYPE: KeyTypeId = KeyTypeId(*b"cer!");

pub mod sr25519 {
    mod app_sr25519 {
        use sp_application_crypto::{app_crypto, sr25519};

        use crate::DAC_VERIFICATION_KEY_TYPE;

        app_crypto!(sr25519, DAC_VERIFICATION_KEY_TYPE);
    }

    sp_application_crypto::with_pair! {
        pub type AuthorityPair = app_sr25519::Pair;
    }
    pub type AuthoritySignature = app_sr25519::Signature;
    pub type AuthorityId = app_sr25519::Public;
}

pub mod crypto {
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify,
        MultiSignature, MultiSigner,
    };

    use crate::DAC_VERIFICATION_KEY_TYPE;

    app_crypto!(sr25519, DAC_VERIFICATION_KEY_TYPE);
    pub struct OffchainIdentifierId;
    impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OffchainIdentifierId {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }

    // implemented for mock runtime in test
    impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
        for OffchainIdentifierId
    {
        type RuntimeAppPublic = Public;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[derive(Default)]
pub struct PayoutReceiptParams {
    pub cluster_id: ClusterId,
    pub era: EhdEra,
    pub state: PayoutState,
    pub fingerprint: Fingerprint,
    pub total_collected_charges: u128,
    pub total_distributed_rewards: u128,
    pub total_settled_fees: u128,
    pub charging_max_batch_index: BatchIndex,
    pub charging_processed_batches: Vec<BatchIndex>,
    pub rewarding_max_batch_index: BatchIndex,
    pub rewarding_processed_batches: Vec<BatchIndex>,
}

#[derive(Default)]
pub struct PayoutFingerprintParams<AccountId> {
    pub cluster_id: ClusterId,
    pub era: EhdEra,
    pub inspection_hash: H256,
    pub ehd_merkle_root: H256,
    pub payers_merkle_root: PayableUsageHash,
    pub payees_merkle_root: PayableUsageHash,
    pub validators: BTreeSet<AccountId>,
}

#[allow(unused)]
#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, DecodeWithMemTracking, TypeInfo, PartialEq)]
pub enum AggregateKey {
    NodeAggregateKey(NodePubKey),
    BucketAggregateKey(BucketId),
    BucketSubAggregateKey(BucketId, NodePubKey),
}

pub fn try_hex_from_string(value: String) -> Result<H256, &'static str> {
    if !value.starts_with("0x") || value.len() != 66 {
        return Err("Input must be a 32-byte hex string started with '0x'");
    }

    let hex_str = &value[2..]; // skip '0x'
    let hex_bytes = match hex::decode(hex_str) {
        Ok(bytes) => bytes,
        Err(_) => return Err("Input must be a valid hex string"),
    };
    if hex_bytes.len() != 32 {
        return Err("Input must be 32 chars in length");
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hex_bytes[..32]);

    Ok(H256::from(hash))
}

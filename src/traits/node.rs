#[cfg(feature = "runtime-benchmarks")]
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

use crate::{ClusterId, NodeParams, NodePubKey};

pub trait NodeManager<AccountId> {
    fn get_cluster_id(node_pub_key: &NodePubKey) -> Result<Option<ClusterId>, DispatchError>;
    fn exists(node_pub_key: &NodePubKey) -> bool;
    fn get_node_provider_id(node_pub_key: &NodePubKey) -> Result<AccountId, DispatchError>;
    fn get_node_params(node_pub_key: &NodePubKey) -> Result<NodeParams, DispatchError>;
    #[cfg(feature = "runtime-benchmarks")]
    fn create_node(
        node_pub_key: NodePubKey,
        provider_id: AccountId,
        node_params: NodeParams,
    ) -> DispatchResult;
}

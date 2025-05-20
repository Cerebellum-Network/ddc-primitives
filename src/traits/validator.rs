use frame_system::Config;
use scale_info::prelude::vec::Vec;
#[cfg(feature = "runtime-benchmarks")]
use sp_runtime::DispatchError;
use sp_runtime::Percent;

pub trait InspectorAuthority<T: Config> {
    fn is_inspector(caller: T::AccountId) -> bool;
    fn is_quorum_reached(quorum: Percent, members_count: usize) -> bool;

    #[cfg(feature = "runtime-benchmarks")]
    fn add_inspector(valdator: T::AccountId) -> Result<(), DispatchError>;
}

pub trait InspReceiptsInterceptor {
    type Receipt;
    fn intercept(receipts: Vec<Self::Receipt>) -> Vec<Self::Receipt>;
}

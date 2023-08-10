use crate::types::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

include!(concat!(env!("OUT_DIR"), "/config.rs"));

// Static stable memory allocation
// We reserve the range [0,64) for our framework.
const GLOBAL_FLAGS_PAGE_SIZE: u64 = 1;
const CANISTER_LIFECYCLE_PAGE_SIZE: u64 = 1;
const ACCESS_CONTROL_PAGE_SIZE: u64 = 4;

pub(crate) const GLOBAL_FLAGS_PAGE_START: u64 = 0;
pub(crate) const GLOBAL_FLAGS_PAGE_END: u64 = GLOBAL_FLAGS_PAGE_START + GLOBAL_FLAGS_PAGE_SIZE;
pub(crate) const CANISTER_LIFECYCLE_PAGE_START: u64 = GLOBAL_FLAGS_PAGE_END;
pub(crate) const CANISTER_LIFECYCLE_PAGE_END: u64 =
    CANISTER_LIFECYCLE_PAGE_START + CANISTER_LIFECYCLE_PAGE_SIZE;
pub(crate) const ACCESS_CONTROL_PAGE_START: u64 = CANISTER_LIFECYCLE_PAGE_END;
pub(crate) const ACCESS_CONTROL_PAGE_END: u64 =
    ACCESS_CONTROL_PAGE_START + ACCESS_CONTROL_PAGE_SIZE;

// Define user page range
pub const USER_PAGE_START: u64 = 64;

// Dynamic stable memory allocation
// MemoryId is u8, and we reserve the range [224,256) for our framework.
// Dynamic stable memory still needs to be instantiated by the user using the `MEMORY_MAMAGER`.
pub(crate) const REENTRANCY_GUARD_MEM_ID: MemoryId = MemoryId::new(224);
//pub(crate) const CANISTER_LIFECYCLE_MEM_ID: MemoryId = MemoryId::new(225);
//pub(crate) const STABLE_LOG_MEM_ID: MemoryId = MemoryId::new(226);
//pub(crate) const STABLE_TRACE_MEM_ID: MemoryId = MemoryId::new(227);
pub const UPGRADE_BUFFER_MEM_ID: MemoryId = MemoryId::new(228);
pub const ACCESS_ROLES_MEM_ID: MemoryId = MemoryId::new(229);

thread_local! {
    // This would be automatically initialized on the first access of anything using the MemoryManager.
    pub static MEMORY_MANAGER: RefCell<MemoryManager<RM>> = RefCell::new(
        MemoryManager::init(RM::new(DefaultMemoryImpl::default(), USER_PAGE_END..u64::MAX/65536-1))
    );
}

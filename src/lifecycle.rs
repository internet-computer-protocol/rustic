#![cfg(feature = "lifecycle")]
// Canister Lifecycle Management
use crate::default_memory_map::*;
use crate::types::*;
use crate::utils::*;
use candid::{candid_method, CandidType};
use ic_cdk_macros::query;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};
use std::cell::RefCell;
#[cfg(test)]
use crate::testing::*;

#[derive(Default, Clone, CandidType, serde::Serialize, serde::Deserialize)]
pub struct CanisterLifecycle {
    stable_memory_version: u16,
    version_major: u16,
    version_minor: u16,
    version_patch: u16,
    last_upgraded: u64,
    ic_canister_version: u64,
}

impl std::fmt::Display for CanisterLifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // "v0.1.15,ic_v24,mem_v3,s.ns"
        write!(
            f,
            "v{}.{}.{},ic_v{},mem_v{},{}.{}",
            self.version_major,
            self.version_minor,
            self.version_patch,
            self.ic_canister_version,
            self.stable_memory_version,
            self.last_upgraded / 1000000000,
            self.last_upgraded % 1000000000,
        )
    }
}

thread_local! {
    pub(crate) static CANISTER_LIFECYCLE: RefCell<StableCell<Cbor<Option<CanisterLifecycle>>, RM>> =
        #[allow(clippy::expect_used)] // safe unwrap
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), CANISTER_LIFECYCLE_PAGE_START..CANISTER_LIFECYCLE_PAGE_END),
            Cbor(Some(CanisterLifecycle{
                stable_memory_version: 0,
                version_major: 0,
                version_minor: 0,
                version_patch: 0,
                last_upgraded: canister_time(),
                ic_canister_version: canister_version(),
            })),
        ).expect("Failed to initialize the canister lifecycle cell")
    );
}

pub(crate) fn canister_lifecycle_init() {
    CANISTER_LIFECYCLE.with(|l| {
        l.borrow();
    });
}

/// Function to be called in the post upgrade hook
pub fn lifecycle_on_upgrade(stable_memory_bump: bool, major_bump: bool, minor_bump: bool) {
    CANISTER_LIFECYCLE.with(|l| {
        let mut l = l.borrow_mut();
        #[allow(clippy::unwrap_used)] // safe unwrap
        let mut lifecycle = l.get().0.clone().unwrap();
        lifecycle.stable_memory_version += if stable_memory_bump { 1 } else { 0 };
        lifecycle.version_patch += 1;
        if minor_bump {
            lifecycle.version_minor += 1;
            lifecycle.version_patch = 0;
        }
        if major_bump {
            lifecycle.version_major += 1;
            lifecycle.version_minor = 0;
            lifecycle.version_patch = 0;
        }
        lifecycle.last_upgraded = canister_time();
        lifecycle.ic_canister_version = canister_version();
        #[allow(clippy::expect_used)] // unwrap desired
        l.set(Cbor(Some(lifecycle)))
            .expect("Lifecycle update failed");
    });
}

#[query]
#[candid_method(query)]
pub fn get_version() -> CanisterLifecycle {
    #[allow(clippy::unwrap_used)] // safe unwrap
    CANISTER_LIFECYCLE.with(|l| l.borrow().get().0.clone().unwrap())
}

#[query]
#[candid_method(query)]
pub fn get_version_text() -> String {
    #[allow(clippy::unwrap_used)] // safe unwrap
    CANISTER_LIFECYCLE.with(|l| l.borrow().get().0.clone().unwrap().to_string())
}

// TODO: stablelog all versions and code and init params

#[cfg(test)]
mod unit_tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_version_patch_bump() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        canister_lifecycle_init();
        assert!(get_version_text().contains("v0.0.0,ic_v0,mem_v0,"));
        lifecycle_on_upgrade(false, false, false);
        assert!(get_version_text().contains("v0.0.1,ic_v0,mem_v0,"));
        lifecycle_on_upgrade(false, false, false);
        assert!(get_version_text().contains("v0.0.2,ic_v0,mem_v0,"));
        lifecycle_on_upgrade(true, false, true);
        assert!(get_version_text().contains("v0.1.0,ic_v0,mem_v1,"));
        set_mock_version(5);
        lifecycle_on_upgrade(true, false, true);
        assert!(get_version_text().contains("v0.2.0,ic_v5,mem_v2,"));
        lifecycle_on_upgrade(true, true, true);
        assert!(get_version_text().contains("v1.0.0,ic_v5,mem_v3,"));
        lifecycle_on_upgrade(true, false, true);
        assert!(get_version_text().contains("v1.1.0,ic_v5,mem_v4,"));
        lifecycle_on_upgrade(true, false, false);
        assert!(get_version_text().contains("v1.1.1,ic_v5,mem_v5,"));
    }
}
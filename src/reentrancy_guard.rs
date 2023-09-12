// =============================================
// ReentrancyGuard
// =============================================
// Usage: declare `_guard = ReentrancyGuard::new();` at the beginning of a public-facing update function
// Attention: the name must be `_some_text` and not `_` in order for the drop checker to be properly scoped
// The guard is binding for each calling principal globally for all functions implementing the guard
#![cfg(feature = "reentrancy")]

//! OpenZeppelin style reentrancy guard.
//! 
//! The concurrency model of the IC is different from that of Ethereum, and a different class of reentrancy issues exist.
//! This library's reentrancy guard deals with a specific type of reentrancy issue,
//! which is limited to a single user, across one single or multiple endpoints.
//! 
//! # Examples
//! Basic usage:
//! ```
//! use rustic::reentrancy_guard::ReentrancyGuard;
//! pub fn some_func() {
//!     let _guard = ReentrancyGuard::new();
//! }
//! ```

use crate::default_memory_map::*;
#[cfg(test)]
use crate::testing::*;
use crate::types::*;
use crate::utils::*;
use candid::Principal;
use ic_stable_structures::StableBTreeMap;
use std::cell::RefCell;

pub struct ReentrancyGuard {
    caller: Principal,
}

thread_local! {
    // can be lazily initialized
    static REENTRANCY_GUARD_MAP: RefCell<StableBTreeMap<StablePrincipal, (), VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableBTreeMap::init(
                mm.borrow().get(REENTRANCY_GUARD_MEM_ID)))
        });
}

impl ReentrancyGuard {
    pub fn new() -> Self {
        let caller = canister_caller();
        if REENTRANCY_GUARD_MAP.with(|g| g.borrow().contains_key(&caller.into())) {
            ic_cdk::trap("ReentrancyGuard: reentrant call");
            //panic!("ReentrancyGuard: reentrant call");
        }
        REENTRANCY_GUARD_MAP.with(|g| g.borrow_mut().insert(caller.into(), ()));
        Self { caller }
    }
}

impl Default for ReentrancyGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ReentrancyGuard {
    fn drop(&mut self) {
        REENTRANCY_GUARD_MAP.with(|g| g.borrow_mut().remove(&self.caller.into()));
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    #[should_panic(expected = "trap should only be called inside canisters")]
    fn test_reentrancy_guard_reentrant() {
        let _guard = ReentrancyGuard::new();
        test_reentrancy_guard_reentrant();
    }

    #[test]
    fn test_reentrancy_guard_non_reentrant() {
        let _guard = ReentrancyGuard::new();
    }

    #[test]
    #[should_panic(expected = "trap should only be called inside canisters")]
    fn test_reentrancy_guard_cross_reentrant() {
        let _guard = ReentrancyGuard::new();
        test_reentrancy_guard_non_reentrant();
    }

    // #[test]
    // #[should_panic(expected = "trap should only be called inside canisters")]
    // #[rustic_macros::modifiers("non_reentrant")]
    // fn test_reentrancy_guard_reentrant_macro() {
    //     test_reentrancy_guard_reentrant_macro();
    // }
}

#![cfg(feature = "pausable")]

//! OpenZeppelin style pausable feature.
//!
//! Note that on the IC canisters have system stopping capabilities.
//! This is unrelated to this pausable feature. This feature is entirely implemented on the application level.
//!
//! The pausable feature is especially useful when certain public-facing functions
//! need to be disabled whilst other functions can still be called normally.
//!
//! By default only `admins` can pause/resume.

use crate::access_control::*;
use crate::global_flags::*;
#[cfg(test)]
use crate::testing::*;
use crate::types::*;
use ic_cdk_macros::{query, update};
use rustic_macros::modifiers;

/// Guard method for validating when a canister is not paused.
/// This is typically used in conjunction with the [`modifiers`] macro.
pub fn when_not_paused() -> Result<(), String> {
    #[allow(clippy::unwrap_used)] // unwrap desired
    if GLOBAL_FLAGS.with(|f| f.borrow().get().0.clone().unwrap().paused) {
        Err("Contract is paused".to_string())
    } else {
        Ok(())
    }
}

/// Guard method for validating when a canister is paused.
/// This is typically used in conjunction with the [`modifiers`] macro.
pub fn when_paused() -> Result<(), String> {
    #[allow(clippy::unwrap_used)] // unwrap desired
    if GLOBAL_FLAGS.with(|f| f.borrow().get().0.clone().unwrap().paused) {
        Ok(())
    } else {
        Err("Contract is not paused".to_string())
    }
}

/// Query method to get the current pause status.
#[query]
pub fn is_paused() -> bool {
    #[allow(clippy::unwrap_used)] // unwrap desired
    GLOBAL_FLAGS.with(|f| f.borrow().get().0.clone().unwrap().paused)
}

/// Pauses the canister. Can only be called by admins.
#[update]
#[modifiers("only_admin")]
pub fn pause() {
    GLOBAL_FLAGS.with(|f| {
        let mut f = f.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut flags = f.get().0.clone().unwrap();
        flags.paused = true;
        #[allow(clippy::expect_used)] // unwrap desired
        f.set(Cbor(Some(flags))).expect("Pause failed");
    });
}

/// Resumes the canister. Can only be called by admins.
#[update]
#[modifiers("only_admin")]
pub fn resume() {
    GLOBAL_FLAGS.with(|f| {
        let mut f = f.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut flags = f.get().0.clone().unwrap();
        flags.paused = false;
        #[allow(clippy::expect_used)] // unwrap desired
        f.set(Cbor(Some(flags))).expect("Resume failed");
    });
}

// TODO: add pause/resume from roles

#[cfg(test)]
mod unit_tests {
    use super::*;
    use candid::Principal;

    #[test]
    fn test_pause() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        global_flags_init();
        assert!(!is_paused());
        pause();
        assert!(is_paused());
        resume();
        assert!(!is_paused());
    }
}

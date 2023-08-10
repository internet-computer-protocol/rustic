// =============================================
// Pausable
// =============================================
#![cfg(feature = "pausable")]

use crate::access_control::*;
use crate::global_flags::*;
use crate::types::*;
use rustic_macros::modifiers;
use candid::candid_method;
use ic_cdk_macros::{query, update};
#[cfg(test)]
use crate::testing::*;

pub fn when_not_paused() -> Result<(), String> {
    #[allow(clippy::unwrap_used)] // unwrap desired
    if GLOBAL_FLAGS.with(|f| f.borrow().get().0.clone().unwrap().paused) {
        Err("Contract is paused".to_string())
    } else {
        Ok(())
    }
}

#[candid_method(update)]
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

#[candid_method(update)]
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

#[candid_method(query)]
#[query]
pub fn is_paused() -> bool {
    #[allow(clippy::unwrap_used)] // unwrap desired
    GLOBAL_FLAGS.with(|f| f.borrow().get().0.clone().unwrap().paused)
}

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
#![doc = include_str!("../README.md")]

// Re-export the macros
#[doc(inline)]
pub use rustic_macros::*;

pub mod access_control;
mod global_flags;
pub mod inter_canister;
pub mod lifecycle;
pub mod logging;
pub mod logging_stable;
pub mod memory_map;
pub mod pausable;
pub mod reentrancy_guard;
pub mod testing;
pub mod types;
pub mod utils;

use crate::utils::canister_caller;

/// Initializes the Rustic module. Needs to be called in the init hook of every canister.
/// # Example
/// ```rust
/// # use ic_cdk::init;
/// #[init]
/// pub fn init () {
///    rustic::rustic_init();
///
///    // your own init code
/// }
/// ```
pub fn rustic_init() {
    // The initialization order is very important.
    //crate::default_memory_map::memory_map_init(); // Not needed
    crate::global_flags::global_flags_init();
    #[cfg(feature = "access")]
    crate::access_control::access_init(canister_caller());
    #[cfg(feature = "lifecycle")]
    crate::lifecycle::canister_lifecycle_init();
    #[cfg(feature = "logging")]
    crate::logging::init(true); // Trace always enabled
}

/// Post-upgrade hook for Rustic. Needs to be called in the post-upgrade hook of every canister.
/// # Example
/// ```rust
/// # use ic_cdk::post_upgrade;
/// #[post_upgrade]
/// pub fn post_upgrade () {
///   rustic::rustic_post_upgrade(false, true, false);
///
///  // your own post-upgrade code
/// }
/// ```
pub fn rustic_post_upgrade(
    #[cfg(feature = "lifecycle")] stable_memory_bump: bool,
    #[cfg(feature = "lifecycle")] major_bump: bool,
    #[cfg(feature = "lifecycle")] minor_bump: bool,
) {
    #[cfg(feature = "lifecycle")]
    crate::lifecycle::lifecycle_on_upgrade(stable_memory_bump, major_bump, minor_bump);
}

#[cfg(all(feature = "lifecycle", feature = "export-candid"))]
use crate::lifecycle::CanisterLifecycle;
#[cfg(feature = "export-candid")]
use candid::Principal;
#[cfg(feature = "export-candid")]
ic_cdk::export_candid!();

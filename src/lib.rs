#![doc = include_str!("../README.md")]

// Re-export the macros
#[doc(inline)]
pub use rustic_macros::*;
use utils::canister_caller;

pub mod access_control;
pub mod default_memory_map;
mod global_flags;
pub mod inter_canister;
pub mod lifecycle;
pub mod pausable;
pub mod reentrancy_guard;
pub mod testing;
//pub mod logging;
pub mod types;
pub mod utils;

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
    crate::logging::logging_init(true); // Trace always enabled
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

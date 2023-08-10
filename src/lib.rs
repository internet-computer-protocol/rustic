pub mod access_control;
pub mod default_memory_map;
mod global_flags;
pub mod inter_canister;
pub mod lifecycle;
pub mod pausable;
pub mod reentrancy_guard;
pub mod testing;
pub mod types;
pub mod utils;

pub fn rustic_init() {
    // The initialization order is very important.
    //crate::default_memory_map::memory_map_init(); // Not needed
    crate::global_flags::global_flags_init();
    #[cfg(feature = "access")]
    crate::access_control::access_init();
    #[cfg(feature = "lifecycle")]
    crate::lifecycle::canister_lifecycle_init();
    #[cfg(feature = "logging")]
    crate::logging::logging_init(true); // Trace always enabled
}

pub fn rustic_post_upgrade(
    #[cfg(feature = "lifecycle")] stable_memory_bump: bool,
    #[cfg(feature = "lifecycle")] major_bump: bool,
    #[cfg(feature = "lifecycle")] minor_bump: bool,
) {
    #[cfg(feature = "lifecycle")]
    crate::lifecycle::lifecycle_on_upgrade(stable_memory_bump, major_bump, minor_bump);
}

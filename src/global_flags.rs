use crate::memory_map::*;
use crate::types::*;
use candid::CandidType;
use ic_cdk_macros::query;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};
use std::cell::RefCell;

include!(concat!(env!("OUT_DIR"), "/config.rs"));

#[derive(Clone, CandidType, serde::Serialize, serde::Deserialize)]
pub(crate) struct GlobalFlags {
    pub(crate) paused: bool,
    pub(crate) logging_initialized: bool,
    pub(crate) log_index: u64,
    pub(crate) trace_index: u64,
    pub(crate) user_page_end: u64,
}

thread_local! {
    pub(crate) static GLOBAL_FLAGS: RefCell<StableCell<Cbor<Option<GlobalFlags>>, RM>> =
        #[allow(clippy::expect_used)] // unwrap desired
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), GLOBAL_FLAGS_PAGE_START..GLOBAL_FLAGS_PAGE_END),
            Cbor(Some(GlobalFlags {
                paused: false,
                logging_initialized: false,
                log_index: 0,
                trace_index: 0,
                user_page_end: USER_PAGE_END,
            })),
        ).expect("Failed to initialize the global flag cell")
    );
}

pub(crate) fn global_flags_init() {
    GLOBAL_FLAGS.with(|gf| {
        gf.borrow();
    });
}

/// Returns the `RUSTIC_USER_PAGE_END` constant used during setup.
/// This value is set through a environment variable and shall remain constant across versions.
#[query]
pub async fn get_config_user_page_end() -> u64 {
    GLOBAL_FLAGS.with(|gf| {
        #[allow(clippy::unwrap_used)] // safe unwrap
        gf.borrow().get().0.clone().unwrap().user_page_end
    })
}

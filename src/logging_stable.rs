// =============================================
// Stable Logging Module for Canisters
// =============================================
#![cfg(feature = "stable-logging")]

//! # Stable Logging Module for Canisters
//! Logs are stored in stable memory, which is persisted across canister upgrades.

use crate::memory_map::*;
#[cfg(test)]
use crate::testing::*;
use crate::types::*;
use crate::utils::*;

use candid::CandidType;
use ic_stable_structures::{storable::Blob, Log, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io::Write;
use tracing::Level;

thread_local! {
    pub static STABLE_LOG: RefCell<Log<StableLogEntry, VM, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(Log::new(mm.borrow().get(STABLE_LOG_IDX_ID), mm.borrow().get(STABLE_LOG_MEM_ID)))
    });
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct StableLogEntry {
    pub timestamp: u64,
    pub message: String,
}

impl Storable for StableLogEntry {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.push(self.message.len() as u8);
        bytes.extend_from_slice(self.message.as_bytes());
        bytes.into()
    }

    /// Panics if the bytes are invalid.
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        assert!(bytes.len() >= 8 + 1);
        #[allow(clippy::unwrap_used)] // unwrap allowed
        let timestamp = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let message_length = bytes[8] as usize;
        let message_bytes = &bytes[9..9 + message_length];
        let message = String::from_utf8_lossy(message_bytes).to_string();
        Self { timestamp, message }
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

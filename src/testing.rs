#![cfg(any(test, doc))]
//! Utils for unit testing.
//!
//! In order to use the mock features, use [`utils::canister_caller`] instead of `ic_cdk::api::caller`
//! and [`utils::canister_id`] instead of `ic_cdk::api::id`, etc.

use candid::Principal;
use std::cell::RefCell;

struct MockData {
    caller: Principal,
    id: Principal,
    version: u64,
    canister_balance: u128,
    time: Option<u64>,
    instruction_counter: Option<u64>,
    controllers: Vec<Principal>,
}

impl MockData {
    fn new() -> Self {
        Self {
            caller: Principal::anonymous(),
            id: Principal::from_text(MOCK_ID).unwrap(),
            version: 0,
            canister_balance: 1000000000000,
            time: None,
            instruction_counter: Some(1000000),
            controllers: vec![],
        }
    }
}

thread_local!(static MOCK_DATA: RefCell<MockData> = RefCell::new(MockData::new()));

/// Sets the mock caller for unit testing.
pub fn set_mock_caller(caller: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().caller = caller;
    });
}

/// Sets the mock canister id for unit testing.
pub fn set_mock_id(id: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().id = id;
    });
}

/// Sets the mock system canister version for unit testing.
pub fn set_mock_version(version: u64) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().version = version;
    });
}

/// Sets the mock canister balance for unit testing.
pub fn set_mock_balance(balance: u128) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().canister_balance = balance;
    });
}

/// Sets the mock time for unit testing.
pub fn set_mock_time(time: u64) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().time = Some(time);
    });
}

/// Sets the mock instruction counter for unit testing.
pub fn set_mock_instruction_counter(counter: u64) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().instruction_counter = Some(counter);
    });
}

/// Adds a mock controller for unit testing.
pub fn add_mock_controller(controller: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().controllers.push(controller);
    });
}

/// Removes a mock controller for unit testing.
pub fn remove_mock_controller(controller: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().controllers.retain(|c| *c != controller);
    });
}

/// Gets the mock controller for unit testing.
pub fn mock_caller() -> Principal {
    MOCK_DATA.with(|data| data.borrow().caller.clone())
}

/// Gets the mock canister id for unit testing.
pub fn mock_id() -> Principal {
    MOCK_DATA.with(|data| data.borrow().id.clone())
}

/// Gets the mock system canister version for unit testing.
pub fn mock_version() -> u64 {
    MOCK_DATA.with(|data| data.borrow().version)
}

/// Gets the mock canister balance for unit testing.
pub fn mock_balance() -> u128 {
    MOCK_DATA.with(|data| data.borrow().canister_balance)
}

/// Gets the mock time for unit testing.
pub fn mock_time() -> u64 {
    MOCK_DATA.with(|data| {
        if let Some(time) = data.borrow().time {
            time
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        }
    })
}

/// Gets the mock instruction counter for unit testing.
pub fn mock_instruction_counter() -> u64 {
    MOCK_DATA.with(|data| {
        if let Some(counter) = data.borrow().instruction_counter {
            counter
        } else {
            0
        }
    })
}

/// Checks if the given principal is a mock controller for unit testing.
pub fn is_mock_controller(controller: &Principal) -> bool {
    MOCK_DATA.with(|data| data.borrow().controllers.contains(controller))
}

/// The mock canister id for unit testing.
pub const MOCK_ID: &str = "732uv-piaaa-aaaag-abkoq-cai";
/// The mock canister id for canister 0.
pub const MOCK_CANISTER_0: &str = "a4gq6-oaaaa-aaaab-qaa4q-cai";
/// The mock canister id for canister 1.
pub const MOCK_CANISTER_1: &str = "mxzaz-hqaaa-aaaar-qaada-cai";
/// The mock canister id for canister 2.
pub const MOCK_CANISTER_2: &str = "3e3x2-xyaaa-aaaaq-aaalq-cai";
/// The mock canister id for canister 3.
pub const MOCK_CANISTER_3: &str = "2jvtu-yqaaa-aaaaq-aaama-cai";
/// The mock canister id for EOA 0.
pub const MOCK_USER_0: &str = "4wcqb-taebw-q5ohj-iwsve-rkznz-aep3a-iv7wl-a5ycz-ajkay-3wmxt-eae";
/// The mock canister id for EOA 1.
pub const MOCK_USER_1: &str = "zlyyc-cf25m-zxq5j-xxy2p-2mmg2-5bv7b-4urrf-tmhkd-reoee-exfr6-kae";
/// The mock canister id for EOA 2.
pub const MOCK_USER_2: &str = "z7zpk-qms36-hlrrb-ofttm-4jais-bk2be-kpyi4-s2w4g-mso4z-h72do-gqe";
/// The mock canister id for EOA 3.
pub const MOCK_USER_3: &str = "ocm3z-6jyo2-xypcq-uchk4-7mhnj-q3wyb-slscx-37fbv-zjeuh-5mfqm-eae";

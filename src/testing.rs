#![cfg(test)]
/// Utils for unit testing
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
            id: Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap(),
            version: 0,
            canister_balance: 1000000000000,
            time: None,
            instruction_counter: Some(1000000),
            controllers: vec![],
        }
    }
}

thread_local!(static MOCK_DATA: RefCell<MockData> = RefCell::new(MockData::new()));

pub fn set_mock_caller(caller: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().caller = caller;
    });
}

pub fn set_mock_id(id: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().id = id;
    });
}

pub fn set_mock_version(version: u64) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().version = version;
    });
}

pub fn set_mock_balance(balance: u128) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().canister_balance = balance;
    });
}

pub fn set_mock_time(time: u64) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().time = Some(time);
    });
}

pub fn set_mock_instruction_counter(counter: u64) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().instruction_counter = Some(counter);
    });
}

pub fn add_mock_controller(controller: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().controllers.push(controller);
    });
}

pub fn remove_mock_controller(controller: Principal) {
    MOCK_DATA.with(|data| {
        data.borrow_mut().controllers.retain(|c| *c != controller);
    });
}

pub fn mock_caller() -> Principal {
    MOCK_DATA.with(|data| data.borrow().caller.clone())
}

pub fn mock_id() -> Principal {
    MOCK_DATA.with(|data| data.borrow().id.clone())
}

pub fn mock_version() -> u64 {
    MOCK_DATA.with(|data| data.borrow().version)
}

pub fn mock_balance() -> u128 {
    MOCK_DATA.with(|data| data.borrow().canister_balance)
}

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

pub fn mock_instruction_counter() -> u64 {
    MOCK_DATA.with(|data| {
        if let Some(counter) = data.borrow().instruction_counter {
            counter
        } else {
            0
        }
    })
}

pub fn is_mock_controller(controller: &Principal) -> bool {
    MOCK_DATA.with(|data| data.borrow().controllers.contains(controller))
}

pub const MOCK_CANISTER_0: &str = "a4gq6-oaaaa-aaaab-qaa4q-cai";
pub const MOCK_CANISTER_1: &str = "mxzaz-hqaaa-aaaar-qaada-cai";
pub const MOCK_CANISTER_2: &str = "3e3x2-xyaaa-aaaaq-aaalq-cai";
pub const MOCK_CANISTER_3: &str = "2jvtu-yqaaa-aaaaq-aaama-cai";

pub const MOCK_USER_0: &str = "4wcqb-taebw-q5ohj-iwsve-rkznz-aep3a-iv7wl-a5ycz-ajkay-3wmxt-eae";
pub const MOCK_USER_1: &str = "zlyyc-cf25m-zxq5j-xxy2p-2mmg2-5bv7b-4urrf-tmhkd-reoee-exfr6-kae";
pub const MOCK_USER_2: &str = "z7zpk-qms36-hlrrb-ofttm-4jais-bk2be-kpyi4-s2w4g-mso4z-h72do-gqe";
pub const MOCK_USER_3: &str = "ocm3z-6jyo2-xypcq-uchk4-7mhnj-q3wyb-slscx-37fbv-zjeuh-5mfqm-eae";

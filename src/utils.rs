//! Utility functions.

use candid::Principal;

// Anonymous callers can be dangerous if not properly validated
// example: `#[rustic_macros::modifiers("not_anonymous@caller")]`
#[allow(dead_code)]
fn not_anonymous(principal: Principal) -> Result<(), String> {
    (principal != candid::Principal::anonymous())
        .then_some(())
        .ok_or("Anonymous caller is not allowed".to_string())
}

// Macro for displaying `type_name_of`
#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

// Methods that wrap `ic_cdk::api` to allow for both testing and production use
#[inline]
pub fn canister_caller() -> Principal {
    #[cfg(not(test))]
    return ic_cdk::api::caller();

    #[cfg(test)]
    return super::testing::mock_caller();
}

#[inline]
pub fn canister_id() -> Principal {
    #[cfg(not(test))]
    return ic_cdk::api::id();

    #[cfg(test)]
    return super::testing::mock_id();
}

#[inline]
pub fn canister_version() -> u64 {
    #[cfg(not(test))]
    return ic_cdk::api::canister_version();

    #[cfg(test)]
    return super::testing::mock_version();
}

#[inline]
pub fn canister_time() -> u64 {
    #[cfg(not(test))]
    return ic_cdk::api::time();

    #[cfg(test)]
    return super::testing::mock_time();
}

#[inline]
pub fn canister_balance() -> u64 {
    #[cfg(not(test))]
    return ic_cdk::api::canister_balance();

    #[cfg(test)]
    return super::testing::mock_balance() as u64;
}

#[inline]
pub fn canister_balance128() -> u128 {
    #[cfg(not(test))]
    return ic_cdk::api::canister_balance128();

    #[cfg(test)]
    return super::testing::mock_balance();
}

#[inline]
pub fn instruction_counter() -> u64 {
    #[cfg(not(test))]
    return ic_cdk::api::instruction_counter();

    #[cfg(test)]
    return super::testing::mock_instruction_counter();
}

#[inline]
#[allow(unused_variables)]
pub fn performance_counter(counter_type: u32) -> u64 {
    #[cfg(not(test))]
    return ic_cdk::api::performance_counter(counter_type);

    #[cfg(test)]
    return super::testing::mock_instruction_counter();
}

#[inline]
pub fn is_controller(caller: &Principal) -> bool {
    #[cfg(not(test))]
    return ic_cdk::api::is_controller(caller);

    #[cfg(test)]
    return super::testing::is_mock_controller(caller);
}

#[inline]
pub fn canister_print<S: std::convert::AsRef<str>>(s: S) {
    #[cfg(not(test))]
    ic_cdk::api::print(s);

    #[cfg(test)]
    println!("{}", s.as_ref()); // Does not print anything during `cargo test`
}

#[inline]
pub fn canister_trap(message: &str) {
    #[cfg(not(test))]
    ic_cdk::api::call::reject(message);

    #[cfg(test)]
    panic!("{}", message);
}

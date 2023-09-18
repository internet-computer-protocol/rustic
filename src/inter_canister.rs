//! Utilities for facilitating inter canister calls.

use candid::Principal;
use ic_cdk::api::call::CallResult;
use std::fmt::Debug;

/// Calls a method on another canister.
/// # Example
/// ```rust
/// # use ic_cdk::api::call::CallResult;
/// # use candid::{CandidType, Deserialize, Principal};
/// # use rustic::inter_canister::canister_call;
/// # pub async fn test() {
/// let result: CallResult<u32> = canister_call(
///    Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap(),
///    "greet",
///    42,
///    candid::encode_one,
///    |r| candid::decode_one(r)
/// ).await;
/// # }
/// ```
pub async fn canister_call<A, R, S, D, SError: Debug, DError: Debug>(
    canister_id: Principal,
    method_name: &str,
    args: A,
    serializer: S,
    deserializer: D,
) -> CallResult<R>
where
    S: Fn(A) -> Result<Vec<u8>, SError>,
    D: Fn(&[u8]) -> Result<R, DError>,
{
    canister_call_with_payment(canister_id, method_name, args, serializer, deserializer, 0).await
}

pub async fn canister_call_with_payment<A, R, S, D, SError: Debug, DError: Debug>(
    canister_id: Principal,
    method_name: &str,
    args: A,
    serializer: S,
    deserializer: D,
    cycles: u128,
) -> CallResult<R>
where
    S: Fn(A) -> Result<Vec<u8>, SError>,
    D: Fn(&[u8]) -> Result<R, DError>,
{
    let payload_bytes = prepare_request(canister_id, method_name, args, serializer)?;

    let response =
        ic_cdk::api::call::call_raw128(canister_id, method_name, &payload_bytes, cycles).await;

    process_response(canister_id, method_name, response, deserializer)
}

/// # Panics
/// Panics on SER error
pub fn canister_call_future<'a, A, S, SError: Debug>(
    canister_id: Principal,
    method_name: &str,
    args: A,
    serializer: S,
) -> impl std::future::Future<Output = CallResult<Vec<u8>>> + Send + Sync + 'a
where
    S: Fn(A) -> Result<Vec<u8>, SError>,
{
    #[allow(clippy::unwrap_used)] // unwrap desired
    let payload_bytes = prepare_request(canister_id, method_name, args, serializer).unwrap();

    ic_cdk::api::call::call_raw128(canister_id, method_name, payload_bytes, 0)
}

pub fn canister_notify<A, S, SError: Debug>(
    canister_id: Principal,
    method_name: &str,
    args: A,
    serializer: S,
) -> CallResult<()>
where
    S: Fn(A) -> Result<Vec<u8>, SError>,
{
    let payload_bytes = prepare_request(canister_id, method_name, args, serializer)?;

    ic_cdk::api::call::notify_raw(canister_id, method_name, &payload_bytes, 0)
        .map_err(|e| (e, "Notify error".to_string()))
}

fn prepare_request<S: Fn(T) -> Result<Vec<u8>, E>, T, E: Debug>(
    canister_id: Principal,
    method_name: &str,
    args: T,
    serializer: S,
) -> CallResult<Vec<u8>> {
    serializer(args).map_err(|err| {
        (
            ic_cdk::api::call::RejectionCode::CanisterError,
            format!(
                "SER Error calling {:?} {:?}: {:?}",
                canister_id.to_string(),
                method_name,
                err
            ),
        )
    })
}

fn process_response<D: Fn(&[u8]) -> Result<T, E>, T, E: Debug>(
    canister_id: Principal,
    method_name: &str,
    response: CallResult<Vec<u8>>,
    deserializer: D,
) -> CallResult<T> {
    match response {
        Ok(result) => deserializer(&result).map_err(|err| {
            (
                ic_cdk::api::call::RejectionCode::CanisterError,
                format!(
                    "DES Error calling {:?} {:?}: {:?}",
                    canister_id.to_string(),
                    method_name,
                    err
                ),
            )
        }),
        Err((error_code, error_message)) => Err((error_code, error_message)),
    }
}

// feat: proc macro
// #[CanisterMethod]
// struct {
//     canister_name,
//     canister_id,
//     method_name,
//     args,
//     response,
// }
// canister_name.method_name(args): CallResult<T of response>

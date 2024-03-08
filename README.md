# Rustic Framework

Rustic is a framework for developing canisters on the Internet Computer.

## Design Goals

- Simplicity. A developer should not need to know all implementation details in most cases.
- Follows best practices. This library should incorporate up-to-date know-how in canister development.
- OpenZeppelin style. Most usage patterns from other chains such as ETH should also be implemented here.

## Features

- [x] access: access control equivalent to OpenZeppelin Ownable2Step + admin list
- [x] access-roles: access control equivalent to OpenZeppelin AccessControl
- [ ] audit-events: events for auditing
- [ ] backup: backup data
- [ ] cache: cache frequently read data into heap for performance
- [ ] certified: certified queries
- [ ] factory: canister factories
- [ ] https: https interface to the canister with metrics etc
- [ ] inspection: cycle histogram for update methods
- [x] lifecycle: canister lifecycle management
- [x] logging: canister logging in heap
- [x] stable-logging: canister logging in stable memory
- [x] pausable: equivalent to OpenZeppelin Pausable
- [ ] payment: payment helpers
- [x] reentrancy: equivalent to OpenZeppelin ReentrancyGuard
- [x] testing: helpers for unit testing
- [ ] tokens: fungible and non-fungible tokens

## Usage

### Before you start

Set environment variable `RUSTIC_USER_PAGE_END`. This value should NOT change across upgrades!
Once you set the user page range, you can never change it! Make sure to leave enough space for future upgrades. Be reasonable and don't set the value too high either, as you pay storage fees for the entire page range even if empty.

### Basic Usage

See examples.

### Stable Memory

The Internet Computer canisters have 4 GB of heap memory that is wiped during canister upgrades, plus 96 GB of stable memory that is perserved across canister upgrades. For robustness and upgradability reasons we should *almost always* prefer stable memory.

The Rustic framework provides an easy way to use stable memory using the [`ic-stable-structures`](https://docs.rs/ic-stable-structures/latest/ic_stable_structures/) crate. Rustic uses the following memory map:

- The first 64 pages are reserved for use by Rustic.
- The user pages start from `USER_PAGE_START` which is page 64.
- Users can store data structures with a known bounded size (read: cannot grow indefinitely) in pages until `RUSTIC_USER_PAGE_END` which is defined in an environment variable.
- The remaining memory is managed by the `MEMORY_MANAGER` and can be used for storing unbounded data structures (such as `StableBTreeMap` and `StableVec`).
- `MEMORY_MANAGER` can dynamically allocate memory with `MemoryId`. The `MemoryId` of each data structure must be unique. `MemoryId` range [0,223] is available for users, while range [224,255] is reserved for Rustic.

### Initialization and post-upgrade hooks

The module must be initialized in the init hook of the main application. The Rustic module must be initialized first before everything else.

```rust
# use ic_cdk::init;
#[init]
pub fn init () {
    rustic::rustic_init();

    // init code for your canister
}
```

The module has a post-upgrade method that you must call in your post-upgrade hook.

```rust
# use ic_cdk::post_upgrade;
#[post_upgrade]
pub fn post_upgrade () {
    rustic::rustic_post_upgrade(false, true, false);

    // post upgrade code for your canister
}
```

### Export Candid

The `export-candid` allows candid export using the mechanism introduced in `ic-cdk` [v0.11](https://github.com/dfinity/cdk-rs/blob/main/src/ic-cdk/CHANGELOG.md#0110---2023-09-18). However due to how this mechanism works, the candid needs to be exported twice, once in your application and once from the Rustic library.

To export the candid from Rustic, use Rustic with the `export-candid` feature, and comment out `ic_cdk::export_candid!()` in your main canister to avoid conflicts. Then generate wasm once, and use [`candid-extractor`](https://github.com/dfinity/cdk-rs/tree/main/src/candid-extractor) to extract the candid.

Then to export candid from your main application, disable the `export-candid` feature, add `ic_cdk::export_candid!()` to your main canister, compile and extract candid again.

Manually combine the two candid files to get the final candid for your canister.

### Lifecycle

When using the `lifecycle` feature (enabled by default), in the post-upgrade hook of the new canister, the `lifecycle_on_upgrade` method is called via calling `rustic::rustic_post_upgrade`. For the semver you'll need to specify whether you want a major/minor/patchlevel version bump. If the stable memory layout changed (make sure you test compatibility as this is not checked by rustic), bump the stable memory version. If you bump the major version then the minor/patchlevel are ignored and will be set to start from 0.

### Notes

Do NOT use the pre-upgrade hook in your application EVER. This feature shall be considered deprecated.

## Caveats

1. Update guard is not executed during unit tests (or any calls from within the canister). This behavior differs from Solidity modifier guards.
`#[update(guard = "only_owner")] pub fn transfer_ownership` expands to:

```rust
    # use candid::Principal;
    # use rustic::access_control::only_owner;
    // This exported function contains the guard
    // #[export_name = "canister_update transfer_ownership"]
    fn transfer_ownership_0_() {
        ic_cdk::setup();
        let r: Result<(), String> = only_owner();
        if let Err(e) = r {
            ic_cdk::api::call::reject(&e);
            return;
        }
        ic_cdk::spawn(async {
            let (new_owner,) = ic_cdk::api::call::arg_data(ic_cdk::api::call::ArgDecoderConfig {
                decoding_quota: None,
                skipping_quota: Some(10000usize),
                debug: false,
            });
            let result = transfer_ownership(new_owner);
            ic_cdk::api::call::reply(())
        });
    }
    // This internal function does not contain the guard
    pub fn transfer_ownership(new_owner: Option<Principal>) {
        // implemantation of transfer_ownership
    }
```

In order to have guards that work for both internal and external calls, the `rustic-macros` crate includes a `modifiers` macro that works for both internal and external calls.

1. The access control is purely on the application level. Note that there's also a system level `controller` that could perform canister upgrades.

## Known issues

## License

MIT

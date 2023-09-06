# Rustic Framework
Rustic is a framework for developing canisters on the Internet Computer.

# Design Goals
- Simplicity. A developer should not need to know all implementation details in most cases.
- Follows best practices. This library should incorporate up-to-date know-how in canister development.
- OpenZeppelin style. Most usage patterns from other chains such as ETH should also be implemented here.

# Features
- [x] access: access control equivalent to OpenZeppelin Ownable2Step + admin list
- [x] access-roles: access control equivalent to OpenZeppelin AccessControl
- [ ] audit-events: events for auditing
- [ ] backup: backup data
- [ ] cache: cache for performance
- [ ] certified: certified queries
- [ ] factory: canister factories
- [ ] https: https interface to the canister with metrics etc
- [ ] inspection: cycle histogram for update methods
- [x] lifecycle: canister lifecycle management
- [ ] logging: canister logging
- [x] pausable: equivalent to OpenZeppelin Pausable
- [ ] payment: payment helpers
- [x] reentrancy: equivalent to OpenZeppelin ReentrancyGuard
- [ ] testing: helpers for unit testing
- [ ] tokens: fungible and non-fungible tokens

# Usage
## Before you start
Set environment variable `RUSTIC_USER_PAGE_END`. This value should NOT change across upgrades!
Once you set the user page range, you can never change it! Make sure to leave enough space for future upgrades. Be reasonable and don't set the value too high either, as you pay storage fees for the entire page range even if empty.

## Basic Usage


## Initialization and post-upgrade hooks
The module must be initialized in the init hook of the main application. The rustic module must be initialized first before everything else.

```rust
#[init]
pub fn init () {
    rustic::rustic_init();

    // init code for your canister
}
```

The module has a post-upgrade method that you must call in your post-upgrade hook.
```rust
#[post_upgrade]
pub fn post_upgrade () {
    rustic::rustic_post_upgrade();

    // post upgrade code for your canister
}
```

## Lifecycle
When using the `lifecycle` feature (enabled by default), in the post-upgrade hook of the new canister, the `lifecycle_on_upgrade` method is called via calling `rustic::rustic_post_upgrade`. For the semver you'll need to specify whether you want a major/minor/patchlevel version bump. If the stable memory layout changed (make sure you test compatibility as this is not checked by rustic), bump the stable memory version. If you bump the major version then the minor/patchlevel are ignored and will be set to start from 0. 

## Notes
Do NOT use the pre-upgrade hook in your application EVER. This feature shall be considered deprecated.

# Known issues

# Caveats
1. Update guard is not executed during unit tests (or any calls from within the canister). This behavior differs from Solidity modifier guards.
`#[update(guard = "only_owner")] pub fn transfer_ownership` expands to:
```rust
    /// # This exported function contains the guard
    #[export_name = "canister_update transfer_ownership"]
    fn transfer_ownership_0_() {
        ic_cdk::setup();
        let r: Result<(), String> = only_owner();
        if let Err(e) = r {
            ic_cdk::api::call::reject(&e);
            return;
        }
        ic_cdk::spawn(async {
            let (new_owner,) = ic_cdk::api::call::arg_data();
            let result = transfer_ownership(new_owner);
            ic_cdk::api::call::reply(())
        });
    }
    /// # This internal function does not contain the guard
    pub fn transfer_ownership(new_owner: Option<Principal>) {
        // implemantation of transfer_ownership
    }
```
In order to have guards that work for both internal and external calls, the `rustic-macros` crate includes a `modifiers` macro that works for both internal and external calls.

# License
MIT

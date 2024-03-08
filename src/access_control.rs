#![cfg(feature = "access")]

//! OpenZeppelin style Ownable2Step and AccessControl with owner as role admin.
//!
//! Each canister on the IC can have multiple `controllers`.
//! These are system level admins that can perform priviledged operations on the canister,
//! such as stopping the canister, or performing code upgrades.
//! While it is possible to grant priviledges on the application level based on whether the caller is the controller,
//! such an approach mixes system level and application level concerns, and is not recommended.
//!
//! This Rustic Access Control module allows for a more fine-grained, flexible access control model on the application level.
//!
//! The `owner` and `admins` is enabled by using the `access` feature flag.
//! The `role_admin` is enabled by using the `access-roles` feature flag.
//!
//! # Access model
//! `owner` is the manager of `admins`. Only `owners` can grant and revoke admins.
//! On init, the principal specified in the `init` function is set as both the `owner` and an `admin`.
//! When using the `access` feature only, these two roles can be used as a two-tier access control system
//! where the top tier is a single super admin (the `owner`), and the second tier consists of multiple admins.
//!
//! When using the `access-control` feature, the above `owner`-`admin` relationship is still valid.
//! `admins` is the manager of all roles (excluding itself), and is the only role that can configure the role admins.
//!
//! A `role_admin` can add/revoke principals to that role, but cannot configure the role admins.
//! A fixed number of 32 roles are defined, and each role is represented by a number of `u8` in [0,32).
//! This number was chosen for the most space-efficient implementation, and should be enough for all practical applications.
//! Unused roles can simply be ignored.

/// `grant_admin` may fail if memory page is full.
use crate::memory_map::*;
#[cfg(test)]
use crate::testing::*;
use crate::types::*;
use crate::utils::*;
use candid::{CandidType, Principal};
use ic_cdk_macros::{query, update};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use rustic_macros::modifiers;
use std::cell::RefCell;

#[derive(Clone, CandidType, serde::Serialize, serde::Deserialize)]
struct AccessControl {
    owner: Option<Principal>,
    pending_owner: Option<Principal>,
    admins: Vec<Principal>,
    // bitflag of admins for each role
    // this is the list of all roles that manage a specific role_i (when the corresponding bitflag is set to 1).
    // Role 0: 0 0 0 ... 1 1 0 0 <- role 0 is managed by role 2 and role 3
    // Role 1: 0 0 0 ... 0 0 0 1 <- role 1 is managed by role 0
    // ...
    // Role x: r31 r30 r29 ... r2 r1 r0
    // It's a bad idea to have a role manage itself or have circular management relationships (but this library would allow it nevertheless)
    admins_of_role: [u32; 32],
}

thread_local! {
    static ACCESS_CONTROL: RefCell<StableCell<Cbor<Option<AccessControl>>, RM>> =
        #[allow(clippy::expect_used)] // safe unwrap during init
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), ACCESS_CONTROL_PAGE_START..ACCESS_CONTROL_PAGE_END),
            Cbor(Some(AccessControl {
                owner: Some(canister_caller()),
                pending_owner: None,
                admins: vec![canister_caller()],
                admins_of_role: Default::default(),
            })),
        ).expect("Failed to initialize the access control cell")
    );
}

pub(crate) fn access_init(owner: Principal) {
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        config.owner = Some(owner);
        config.admins = vec![owner];
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config)))
            .expect("Access control init failed");
    });
}

/// Checks if the caller is the owner.
/// This is typically used in conjunction with the [`modifiers`] macro
/// # Example
/// ```rust
/// # use ic_cdk::update;
/// # use rustic::access_control::only_owner;
/// # use rustic_macros::modifiers;
/// #[update]
/// #[modifiers("only_owner")]
/// fn my_func() {}
/// ```
pub fn only_owner() -> Result<(), String> {
    let caller = canister_caller();
    #[allow(clippy::unwrap_used)] // unwrap desired
    if ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().owner) == Some(caller) {
        Ok(())
    } else {
        Err("Caller is not the owner".to_string())
    }
}

/// Checks if a principal is the owner.
#[query]
pub fn is_owner(owner: Principal) -> bool {
    ACCESS_CONTROL.with(|c| {
        #[allow(clippy::unwrap_used)] // unwrap desired
        c.borrow().get().0.clone().unwrap().owner
    }) == Some(owner)
}

/// Transfers ownership to a new Principal in a 2-step transfer process.
/// Must be called by the current `owner`
///
/// First, the original owner calls this function, specifying the new owner.
/// The ownership is not affected until the new owners calls [`accept_ownership`],
/// at which point ownership would be transfered from the original owner to the new owner.
/// If the `new_owner` is set to `None`, then any pending ownership transfer is cancelled.
#[update]
#[modifiers("only_owner")]
pub fn transfer_ownership(new_owner: Option<Principal>) {
    if let Some(x) = new_owner {
        assert_ne!(
            x,
            Principal::anonymous(),
            "Cannot transfer ownership to the anonymous principal"
        );
    }
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        config.pending_owner = new_owner;
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config)))
            .expect("Ownership transfer failed");
    });
}

/// Transfers ownership to a new Principal in a single-step transfer process.
/// Must be called by the current `owner`.
///
/// The use of this function is discouraged, as there is no recourse if the wrong principal is specified.
/// This function is useful in cases where the accepting principal cannot call [`accept_ownership`].
#[update]
#[modifiers("only_owner")]
pub fn transfer_ownership_immediate(new_owner: Option<Principal>) {
    if let Some(x) = new_owner {
        assert_ne!(
            x,
            Principal::anonymous(),
            "Cannot transfer ownership to the anonymous principal"
        );
    }
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        config.pending_owner = None;
        config.owner = new_owner;
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config)))
            .expect("Ownership transfer failed");
    });
}

/// Renounces ownership. Must be called by the current `owner`.
#[update]
#[modifiers("only_owner")]
pub fn renounce_ownership() {
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        config.owner = None;
        config.pending_owner = None;
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config)))
            .expect("Ownership transfer failed");
    });
}

/// Accepts ownership transfer. The caller must be the pending owner.
#[update]
pub fn accept_ownership() {
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        #[allow(clippy::expect_used)] // unwrap desired
        let new_owner = config.pending_owner.expect("No pending owner");
        assert_eq!(
            new_owner,
            canister_caller(),
            "Only pending owner can accept ownership"
        );
        config.owner = Some(new_owner);
        config.pending_owner = None;
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config)))
            .expect("Ownership transfer failed");
    });
}

/// Query method to get the current owner.
#[query]
pub fn owner() -> Option<Principal> {
    #[allow(clippy::unwrap_used)] // unwrap desired
    ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().owner)
}

/// Query method to get the current pending owner.
#[query]
pub fn pending_owner() -> Option<Principal> {
    #[allow(clippy::unwrap_used)] // unwrap desired
    ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().pending_owner)
}

/// Query method to get the current owner and pending owner.
#[query]
pub fn owner_and_pending_owner() -> (Option<Principal>, Option<Principal>) {
    #[allow(clippy::unwrap_used)] // unwrap desired
    let config = ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap());
    (config.owner, config.pending_owner)
}

/// Checks if the caller is the admin.
/// This is typically used in conjunction with the [`modifiers`] macro
/// # Example
/// ```rust
/// # use ic_cdk::update;
/// # use rustic_macros::modifiers;
/// # use rustic::access_control::only_admin;
/// #[update]
/// #[modifiers("only_admin")]
/// fn my_func() {}
/// ```
pub fn only_admin() -> Result<(), String> {
    let caller = canister_caller();
    #[allow(clippy::unwrap_used)] // unwrap desired
    if ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().admins.contains(&caller)) {
        Ok(())
    } else {
        Err("Caller is not an admin".to_string())
    }
}

/// Checks if a principal is an admin.
#[query]
pub fn is_admin(admin: Principal) -> bool {
    #[allow(clippy::unwrap_used)] // unwrap desired
    ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().admins.contains(&admin))
}

/// Grants admin to a new Principal. Must be called by the `owner`.
#[update]
#[modifiers("only_owner")]
pub fn grant_admin(new_admin: Principal) {
    assert_ne!(
        new_admin,
        Principal::anonymous(),
        "Cannot grant admin to the anonymous principal"
    );
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        if !config.admins.contains(&new_admin) {
            config.admins.push(new_admin);
            #[allow(clippy::expect_used)] // unwrap desired
            c.set(Cbor(Some(config))).expect("Grant admin failed");
        }
    });
}

/// Revokes admin from a Principal. Must be called by the `owner`.
#[update]
#[modifiers("only_owner")]
pub fn revoke_admin(admin: Principal) {
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        config.admins.retain(|x| x != &admin);
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config))).expect("Revoke admin failed");
    });
}

/// Revokes admin from the caller. Must be called by the admin itself.
#[update]
#[modifiers("only_admin")]
pub fn renounce_admin() {
    let admin = canister_caller();
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        config.admins.retain(|x| x != &admin);
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config))).expect("Revoke admin failed");
    });
}

// `access-roles` feature

thread_local! {
    // can be lazily initialized
    // mapping from Principal to bitflag of roles
    static ACCESS_ROLES: RefCell<StableBTreeMap<StablePrincipal, u32, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableBTreeMap::init(
                mm.borrow().get(ACCESS_ROLES_MEM_ID)))
    });
}

// If any role in the role flag is a role admin of another role.
// Panics if role index is out of range.
#[cfg(feature = "access-roles")]
fn is_role_admin(role_flag: u32, role: u8) -> bool {
    assert!(role <= 31, "Role must be between 0 and 31");
    ACCESS_CONTROL.with(|c| {
        let c = c.borrow();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let config = c.get().0.clone().unwrap();
        config.admins_of_role[role as usize] & role_flag != 0
    })
}

/// Grants roles to a principal. Must be called by the `owner` or a `role_admin`.
/// Returns a vector of booleans indicating whether each role was successfully granted, in the same order as the input.
///
/// When a role has already been granted prior to calling this function,
/// but the current caller has the permission to grant the role,
/// the return value for that role is `true`.
/// When a role has already been granted prior to calling this function,
/// but the current caller does not have the permission to grant the role,
/// the return value for that role is `false`.
#[cfg(feature = "access-roles")]
#[update]
pub fn grant_roles(roles: Vec<u8>, principal: Principal) -> Vec<bool> {
    // caller authentication in arithmetics
    let mut success = Vec::with_capacity(roles.len());
    ACCESS_ROLES.with(|ar| {
        let mut ar = ar.borrow_mut();
        let mut principal_roles = ar.get(&principal.into()).unwrap_or(0);
        let caller_roles = ar.get(&canister_caller().into()).unwrap_or(0);

        for role in roles {
            if role <= 31 && (is_admin(canister_caller()) || is_role_admin(caller_roles, role)) {
                principal_roles |= 1 << role;
                success.push(true);
            } else {
                success.push(false);
            }
        }
        #[allow(clippy::expect_used)] // unwrap desired
        ar.insert(principal.into(), principal_roles);
    });
    success
}

/// Revokes roles from a principal. Must be called by the `owner` or a `role_admin`.
/// Returns a vector of booleans indicating whether each role was successfully revoked, in the same order as the input.
///
/// When a role has already been revoked prior to calling this function,
/// but the current caller has the permission to revoke the role,
/// the return value for that role is `true`.
/// When a role has already been revoked prior to calling this function,
/// but the current caller does not have the permission to revoke the role,
/// the return value for that role is `false`.
#[cfg(feature = "access-roles")]
#[update]
pub fn revoke_roles(roles: Vec<u8>, principal: Principal) -> Vec<bool> {
    // caller authentication arithmetics
    let mut success = Vec::with_capacity(roles.len());
    ACCESS_ROLES.with(|c| {
        let mut c = c.borrow_mut();
        let mut principal_roles = c.get(&principal.into()).unwrap_or(0);
        let caller_roles = c.get(&canister_caller().into()).unwrap_or(0);

        for role in roles {
            if role <= 31 && (is_admin(canister_caller()) || is_role_admin(caller_roles, role)) {
                principal_roles &= !(1 << role);
                success.push(true);
            } else {
                success.push(false);
            }
        }
        #[allow(clippy::expect_used)] // unwrap desired
        c.insert(principal.into(), principal_roles)
            .expect("Role update failed");
    });
    success
}

/// Returns the bitflag of roles granted to a principal.
#[cfg(feature = "access-roles")]
#[query]
pub fn get_user_roles(principal: Principal) -> u32 {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        c.get(&principal.into()).unwrap_or(0)
    })
}

/// Checks whether a principal has a certain role.
/// Returns a boolean indicating whether the principal has the role.
#[cfg(feature = "access-roles")]
#[query]
pub fn user_has_role(role: u8, principal: Principal) -> bool {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&principal.into()).unwrap_or(0);
        principal_roles & (1 << role) != 0
    })
}

/// Checks whether the caller has a certain role.
/// This is typically used in conjunction with the [`modifiers`] macro.
#[cfg(feature = "access-roles")]
pub fn has_role(role: u8) -> Result<(), String> {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&canister_caller().into()).unwrap_or(0);
        if principal_roles & (1 << role) != 0 {
            Ok(())
        } else {
            Err("Unauthorized".to_string())
        }
    })
}

/// Checks whether a principal has all of the specified roles.
/// Returns a boolean indicating whether the principal has all of the roles.
#[cfg(feature = "access-roles")]
#[query]
pub fn user_has_roles_all(roles: Vec<u8>, principal: Principal) -> bool {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&principal.into()).unwrap_or(0);
        roles.iter().all(|role| principal_roles & (1 << role) != 0)
    })
}

/// Checks whether the caller has all of the specified roles.
/// This is typically used in conjunction with the [`modifiers`] macro.
#[cfg(feature = "access-roles")]
pub fn has_roles_all(roles: Vec<u8>) -> Result<(), String> {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&canister_caller().into()).unwrap_or(0);
        if roles.iter().all(|role| principal_roles & (1 << role) != 0) {
            Ok(())
        } else {
            Err("Unauthorized".to_string())
        }
    })
}

/// Checks whether a principal has any of the specified roles.
/// Returns a boolean indicating whether the principal has any of the roles.
#[cfg(feature = "access-roles")]
#[query]
pub fn user_has_roles_any(roles: Vec<u8>, principal: Principal) -> bool {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&principal.into()).unwrap_or(0);
        roles.iter().any(|role| principal_roles & (1 << role) != 0)
    })
}

/// Checks whether the caller has any of the specified roles.
/// This is typically used in conjunction with the [`modifiers`] macro.
#[cfg(feature = "access-roles")]
pub fn has_roles_any(roles: Vec<u8>) -> Result<(), String> {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&canister_caller().into()).unwrap_or(0);
        if roles.iter().any(|role| principal_roles & (1 << role) != 0) {
            Ok(())
        } else {
            Err("Unauthorized".to_string())
        }
    })
}

/// Sets role admins for a role. Must be called by admins.
#[cfg(feature = "access-roles")]
#[update]
#[modifiers("only_admin")]
pub fn set_role_admins(role: u8, admins: Vec<u8>) {
    let admin_bit_flags = admins.iter().fold(0, |acc, x| acc | (1 << x));
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        if let Some(x) = config.admins_of_role.get_mut(role as usize) {
            *x |= admin_bit_flags;
        }
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config))).expect("Set role admin failed");
    });
}

/// Revokes role admins for a role. Must be called by admins.
#[cfg(feature = "access-roles")]
#[update]
#[modifiers("only_admin")]
pub fn revoke_role_admins(role: u8, admins: Vec<u8>) {
    let admin_bit_flags = admins.iter().fold(0, |acc, x| acc | (1 << x));
    ACCESS_CONTROL.with(|c| {
        let mut c = c.borrow_mut();
        #[allow(clippy::unwrap_used)] // unwrap desired
        let mut config = c.get().0.clone().unwrap();
        if let Some(x) = config.admins_of_role.get_mut(role as usize) {
            *x &= !admin_bit_flags;
        }
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config))).expect("Revoke role admin failed");
    });
}

#[cfg(test)]
mod access_tests {
    use super::*;

    #[test]
    fn test_ownable() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(only_owner().is_ok());
        assert!(owner() == Principal::from_text(MOCK_USER_0).ok());
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        assert!(only_owner().is_err());
    }

    #[test]
    fn test_ownable_transfer_ownership() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(only_owner().is_ok());
        assert!(owner() == Principal::from_text(MOCK_USER_0).ok());
        transfer_ownership(Some(Principal::from_text(MOCK_USER_1).unwrap()));
        assert!(owner() == Principal::from_text(MOCK_USER_0).ok());
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        assert!(only_owner().is_err());
        accept_ownership();
        assert!(only_owner().is_ok());
        assert!(owner() == Principal::from_text(MOCK_USER_1).ok());
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        assert!(only_owner().is_err());
    }

    #[test]
    #[should_panic(expected = "No pending owner")]
    fn test_ownable_transfer_ownership_to_none() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(only_owner().is_ok());
        transfer_ownership(None);
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        accept_ownership();
    }

    // Test failes due to [caveat1]
    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    fn test_ownable_unauth_transfer() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(only_owner().is_ok());
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        transfer_ownership(Some(Principal::from_text(MOCK_USER_1).unwrap()));
    }

    #[test]
    fn test_ownable_transfer_ownership_to_self() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(only_owner().is_ok());
        transfer_ownership(Some(Principal::from_text(MOCK_USER_0).unwrap()));
        assert!(only_owner().is_ok());
        accept_ownership();
        assert!(only_owner().is_ok());
    }

    #[test]
    fn test_admin() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(only_admin().is_ok());
        assert!(is_admin(Principal::from_text(MOCK_USER_0).unwrap()));
        assert!(!is_admin(Principal::from_text(MOCK_USER_1).unwrap()));
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        assert!(only_admin().is_err());
    }

    #[test]
    fn test_grant_admin() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(!is_admin(Principal::from_text(MOCK_USER_1).unwrap()));
        grant_admin(Principal::from_text(MOCK_USER_1).unwrap());
        assert!(is_admin(Principal::from_text(MOCK_USER_1).unwrap()));
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        assert!(only_admin().is_ok());
    }

    #[test]
    fn test_revoke_admin() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        assert!(is_admin(Principal::from_text(MOCK_USER_0).unwrap()));
        revoke_admin(Principal::from_text(MOCK_USER_0).unwrap());
        assert!(!is_admin(Principal::from_text(MOCK_USER_0).unwrap()));
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        assert!(only_admin().is_err());
    }

    #[test]
    fn test_renounce_admin() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        grant_admin(Principal::from_text(MOCK_USER_1).unwrap());
        assert!(is_admin(Principal::from_text(MOCK_USER_1).unwrap()));
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        assert!(only_admin().is_ok());
        renounce_admin();
        assert!(!is_admin(Principal::from_text(MOCK_USER_1).unwrap()));
    }

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    fn grant_admin_unauth() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        grant_admin(Principal::from_text(MOCK_USER_1).unwrap());
    }

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    fn revoke_admin_unauth() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
        set_mock_caller(Principal::from_text(MOCK_USER_1).unwrap());
        revoke_admin(Principal::from_text(MOCK_USER_1).unwrap());
    }
}

#[cfg(test)]
#[cfg(feature = "access-roles")]
mod access_role_tests {
    use super::*;

    #[repr(u8)]
    enum Role {
        R0 = 0,
        R1 = 1,
        R2 = 2,
        R3 = 3,
    }

    impl From<Role> for u8 {
        fn from(role: Role) -> Self {
            role as u8
        }
    }

    const ROLE0: u8 = 0;
    const ROLE1: u8 = 1;
    #[allow(unused)]
    const ROLE2: u8 = 2;
    #[allow(unused)]
    const ROLE3: u8 = 3;

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    #[modifiers("has_role@Role::R0.into()")]
    fn test_syntax_role_modifiers_enum() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
    }

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    #[modifiers("has_role@ROLE0")]
    fn test_syntax_role_modifiers_const() {
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());
    }

    // thread 'access_control::access_role_tests::test_grant_roles' panicked at 'Role update failed', src/access_control.rs:226:14
    #[test]
    fn test_grant_roles() {
        use std::collections::BTreeMap;
        set_mock_caller(Principal::from_text(MOCK_USER_0).unwrap());
        access_init(canister_caller());

        assert!(!user_has_role(
            Role::R0.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));
        assert!(!user_has_role(
            Role::R1.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));
        assert!(!user_has_role(
            Role::R2.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));
        assert!(!user_has_role(
            Role::R3.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));

        assert!(is_admin(canister_caller()));

        grant_roles(
            vec![Role::R0.into(), Role::R1.into()],
            Principal::from_text(MOCK_USER_1).unwrap(),
        );
        grant_roles(
            vec![ROLE0, ROLE1],
            Principal::from_text(MOCK_USER_1).unwrap(),
        );

        assert!(user_has_role(
            Role::R0.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));
        assert!(user_has_role(
            Role::R1.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));
        assert!(!user_has_role(
            Role::R2.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));
        assert!(!user_has_role(
            Role::R3.into(),
            Principal::from_text(MOCK_USER_1).unwrap()
        ));

        let mut test_btreemap: BTreeMap<u32, u32> = BTreeMap::new();
        let first_insertion = test_btreemap.insert(12u32, 4u32);
        let second_insertion = test_btreemap.insert(12u32, 7u32).expect("insert error");
        assert_eq!(first_insertion, None);
        assert_eq!(second_insertion, 4);
        assert_eq!(test_btreemap.get(&12u32), Some(7).as_ref());
    }
}

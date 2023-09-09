#![cfg(feature = "access")]

/// # Ownable and AccessControl
/// OpenZeppelin style Ownable2Step and AccessControl with owner as role admin.
/// `grant_admin` may fail if memory page is full.
/// `owner` is the manager of `admins`.
/// `admins` is the manager of all roles (excluding itself).
/// `role_admins[i]` is the list of all roles that manage a specific role_i.
/// A `role_admin` can add/revoke other principals, but cannot configure role admins for that role.
/// if a role is defined in its own admin list, then it can manage itself.
/// A maximum of 32 roles can be defined, each role is represented by a number of `u8`.


use crate::default_memory_map::*;
#[cfg(test)]
use crate::testing::*;
use crate::types::*;
use crate::utils::*;
use candid::{candid_method, CandidType, Principal};
use ic_cdk_macros::{query, update};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use rustic_macros::modifiers;
use std::cell::RefCell;

#[derive(Clone, CandidType, serde::Serialize, serde::Deserialize)]
struct AccessControl {
    owner: Principal,
    pending_owner: Option<Principal>,
    admins: Vec<Principal>,
    // bitflag of admins for each role
    // Role 0: 0 0 0 ... 1 1 0 0 <- role 0 is managed by role 2 and role 3
    // Role 1: 0 0 0 ... 0 0 0 1 <- role 1 is managed by role 0
    // It's a bad idea to have a role manage itself or have circular management (but this library would allow it)
    admins_of_role: [u32; 32],
}

thread_local! {
    static ACCESS_CONTROL: RefCell<StableCell<Cbor<Option<AccessControl>>, RM>> =
        #[allow(clippy::expect_used)] // safe unwrap during init
        RefCell::new(StableCell::init(
            RM::new(DefaultMemoryImpl::default(), ACCESS_CONTROL_PAGE_START..ACCESS_CONTROL_PAGE_END),
            Cbor(Some(AccessControl {
                owner: canister_caller(),
                pending_owner: None,
                admins: vec![canister_caller()],
                admins_of_role: Default::default(),
            })),
        ).expect("Failed to initialize the access control cell")
    );
}

pub(crate) fn access_init() {
    ACCESS_CONTROL.with(|c| c.borrow().get().0.clone());
}

/// Method to be used to check if the caller is the owner.
/// This is typically used in conjunction with the `modifiers` macro
/// # Example
/// ```rust
/// #[update]
/// #[modifiers("only_owner")]
/// fn my_func() {}
/// ```
pub fn only_owner() -> Result<(), String> {
    let caller = canister_caller();
    #[allow(clippy::unwrap_used)] // unwrap desired
    if ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().owner) == caller {
        Ok(())
    } else {
        Err("Caller is not the owner".to_string())
    }
}

/// Transfer ownership to a new Principal in a 2-step transfer process.
/// First, the original owner calls this function, specifying the new owner.
/// The ownership is not affected until the new owners calls [`accept_ownership`s],
/// at which point ownership would be transfered from the original owner to the new owner.
/// If the `new_owner` is set to `None`, then the ownership transfer is cancelled.
/// [`accept_ownership`]: #method.accept_ownership
#[candid_method(update)]
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

/// Accepts ownership transfer.
#[candid_method(update)]
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
        config.owner = new_owner;
        config.pending_owner = None;
        #[allow(clippy::expect_used)] // unwrap desired
        c.set(Cbor(Some(config)))
            .expect("Ownership transfer failed");
    });
}

#[candid_method(query)]
#[query]
pub fn owner() -> Principal {
    #[allow(clippy::unwrap_used)] // unwrap desired
    ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().owner)
}

/// Method to be used in `guard` to check if the caller is an admin

pub fn only_admin() -> Result<(), String> {
    let caller = canister_caller();
    #[allow(clippy::unwrap_used)] // unwrap desired
    if ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().admins.contains(&caller)) {
        Ok(())
    } else {
        Err("Caller is not an admin".to_string())
    }
}

#[candid_method(update)]
#[update]
#[modifiers("only_owner")]
pub fn grant_admin(new_admin: Principal) {
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

#[candid_method(update)]
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

#[candid_method(update)]
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

#[candid_method(query)]
#[query]
pub fn is_admin(admin: Principal) -> bool {
    #[allow(clippy::unwrap_used)] // unwrap desired
    ACCESS_CONTROL.with(|c| c.borrow().get().0.clone().unwrap().admins.contains(&admin))
}

#[cfg(feature = "access-roles")]
thread_local! {
    // can be lazily initialized
    static ACCESS_ROLES: RefCell<StableBTreeMap<StablePrincipal, u32, VM>> =
        MEMORY_MANAGER.with(|mm| {
            RefCell::new(StableBTreeMap::init(
                mm.borrow().get(ACCESS_ROLES_MEM_ID)))
    });
}

#[cfg(feature = "access-roles")]
#[candid_method(update)]
#[update]
pub fn grant_roles(roles: Vec<u8>, principal: Principal) -> Vec<bool> {
    // caller authentication in arithmetics
    let mut success = Vec::with_capacity(roles.len());
    ACCESS_ROLES.with(|c| {
        let mut c = c.borrow_mut();
        let mut principal_roles = c.get(&principal.into()).unwrap_or(0);
        let caller_roles = c.get(&canister_caller().into()).unwrap_or(0);
        let is_role_admin = |role: u8| {
            ACCESS_CONTROL.with(|ac| {
                let ac = ac.borrow();
                #[allow(clippy::unwrap_used)] // unwrap desired
                let config = ac.get().0.clone().unwrap();
                config
                    .admins_of_role
                    .get(1 << role)
                    .map_or(false, |x| caller_roles & x != 0)
            })
        };
        for role in roles {
            if role <= 31 && (is_admin(canister_caller()) || is_role_admin(role)) {
                principal_roles |= 1 << role;
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

#[cfg(feature = "access-roles")]
#[candid_method(update)]
#[update]
pub fn revoke_roles(roles: Vec<u8>, principal: Principal) -> Vec<bool> {
    // caller authentication arithmetics
    let mut success = Vec::with_capacity(roles.len());
    ACCESS_ROLES.with(|c| {
        let mut c = c.borrow_mut();
        let mut principal_roles = c.get(&principal.into()).unwrap_or(0);
        let caller_roles = c.get(&canister_caller().into()).unwrap_or(0);
        let is_role_admin = |role: u8| {
            ACCESS_CONTROL.with(|ac| {
                let ac = ac.borrow();
                #[allow(clippy::unwrap_used)] // unwrap desired
                let config = ac.get().0.clone().unwrap();
                config
                    .admins_of_role
                    .get(1 << role)
                    .map_or(false, |x| caller_roles & x != 0)
            })
        };
        for role in roles {
            if role <= 31 && (is_admin(canister_caller()) || is_role_admin(role)) {
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

#[cfg(feature = "access-roles")]
#[candid_method(query)]
#[query]
pub fn user_has_role(role: u8, principal: Principal) -> bool {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&principal.into()).unwrap_or(0);
        principal_roles & (1 << role) != 0
    })
}

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

#[cfg(feature = "access-roles")]
#[candid_method(query)]
#[query]
pub fn user_has_roles_all(roles: Vec<u8>, principal: Principal) -> bool {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&principal.into()).unwrap_or(0);
        roles.iter().all(|role| principal_roles & (1 << role) != 0)
    })
}

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

#[cfg(feature = "access-roles")]
#[candid_method(query)]
#[query]
pub fn user_has_roles_any(roles: Vec<u8>, principal: Principal) -> bool {
    ACCESS_ROLES.with(|c| {
        let c = c.borrow();
        let principal_roles = c.get(&principal.into()).unwrap_or(0);
        roles.iter().any(|role| principal_roles & (1 << role) != 0)
    })
}

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

#[cfg(feature = "access-roles")]
#[candid_method(update)]
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

#[cfg(feature = "access-roles")]
#[candid_method(update)]
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
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(only_owner().is_ok());
        assert!(owner() == Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        assert!(only_owner().is_err());
    }

    #[test]
    fn test_ownable_transfer_ownership() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(only_owner().is_ok());
        assert!(owner() == Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        transfer_ownership(Some(
            Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap(),
        ));
        assert!(owner() == Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        assert!(only_owner().is_err());
        accept_ownership();
        assert!(only_owner().is_ok());
        assert!(owner() == Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        assert!(only_owner().is_err());
    }

    #[test]
    #[should_panic(expected = "No pending owner")]
    fn test_ownable_transfer_ownership_to_none() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(only_owner().is_ok());
        transfer_ownership(None);
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        accept_ownership();
    }

    // Test failes due to [known issue 1]
    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    fn test_ownable_unauth_transfer() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(only_owner().is_ok());
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        transfer_ownership(Some(
            Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap(),
        ));
    }

    #[test]
    fn test_ownable_transfer_ownership_to_self() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(only_owner().is_ok());
        transfer_ownership(Some(
            Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap(),
        ));
        assert!(only_owner().is_ok());
        accept_ownership();
        assert!(only_owner().is_ok());
    }

    #[test]
    fn test_admin() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(only_admin().is_ok());
        assert!(is_admin(
            Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap()
        ));
        assert!(!is_admin(
            Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()
        ));
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        assert!(only_admin().is_err());
    }

    #[test]
    fn test_grant_admin() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(!is_admin(
            Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()
        ));
        grant_admin(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        assert!(is_admin(
            Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()
        ));
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        assert!(only_admin().is_ok());
    }

    #[test]
    fn test_revoke_admin() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        assert!(is_admin(
            Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap()
        ));
        revoke_admin(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        assert!(!is_admin(
            Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap()
        ));
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        assert!(only_admin().is_err());
    }

    #[test]
    fn test_renounce_admin() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        grant_admin(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        assert!(is_admin(
            Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()
        ));
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        assert!(only_admin().is_ok());
        renounce_admin();
        assert!(!is_admin(
            Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()
        ));
    }

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    fn grant_admin_unauth() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        grant_admin(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
    }

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    fn revoke_admin_unauth() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
        set_mock_caller(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
        revoke_admin(Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());
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

    impl std::convert::Into<u8> for Role {
        fn into(self) -> u8 {
            self as u8
        }
    }

    const ROLE0: u8 = 0;
    const ROLE1: u8 = 1;
    const ROLE2: u8 = 2;
    const ROLE3: u8 = 3;

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    #[modifiers("has_role@Role::R0.into()")]
    fn test_syntax_role_modifiers_enum() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
    }

    #[test]
    #[should_panic(expected = "msg_reject should only be called inside canisters")]
    #[modifiers("has_role@ROLE0")]
    fn test_syntax_role_modifiers_const() {
        set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
        access_init();
    }

    // thread 'access_control::access_role_tests::test_grant_roles' panicked at 'Role update failed', src/access_control.rs:226:14
    // #[test]
    // fn test_grant_roles(){
    //     set_mock_caller(Principal::from_text("a4gq6-oaaaa-aaaab-qaa4q-cai").unwrap());
    //     access_init();

    //     assert!(!user_has_role(Role::R0.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));
    //     assert!(!user_has_role(Role::R1.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));
    //     assert!(!user_has_role(Role::R2.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));
    //     assert!(!user_has_role(Role::R3.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));

    //     grant_roles(vec![Role::R0.into(),Role::R1.into()], Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap());

    //     assert!(user_has_role(Role::R0.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));
    //     assert!(user_has_role(Role::R1.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));
    //     assert!(!user_has_role(Role::R2.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));
    //     assert!(!user_has_role(Role::R3.into(), Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap()));
    // }
}

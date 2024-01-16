//! Type definitions.
use candid::Principal;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Blob;
use ic_stable_structures::{DefaultMemoryImpl, RestrictedMemory, Storable};
use std::borrow::Cow;

pub type RM = RestrictedMemory<DefaultMemoryImpl>;
pub type VM = VirtualMemory<RM>;

/// A helper type implementing Storable for all
/// serde-serializable types using the CBOR encoding.
#[derive(Default)]
pub struct Cbor<T>(pub T)
where
    T: serde::Serialize + serde::de::DeserializeOwned;

impl<T> std::ops::Deref for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// # Panics
/// Panics if the serialization/deserialization fails.
impl<T> Storable for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        #[allow(clippy::unwrap_used)] // unwrap expected
        ciborium::ser::into_writer(&self.0, &mut buf).unwrap();
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        #[allow(clippy::unwrap_used)] // unwrap expected
        Self(ciborium::de::from_reader(bytes.as_ref()).unwrap())
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

/// Stable storage for Principal
/// # Panics
/// Panics if the serialization/deserialization fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StablePrincipal(Blob<29>);

impl From<Principal> for StablePrincipal {
    fn from(caller: Principal) -> Self {
        #[allow(clippy::unwrap_used)] // unwrap expected
        Self(Blob::try_from(caller.as_slice()).unwrap())
    }
}

impl From<&Principal> for StablePrincipal {
    fn from(caller: &Principal) -> Self {
        #[allow(clippy::unwrap_used)] // unwrap expected
        Self(Blob::try_from(caller.as_slice()).unwrap())
    }
}

impl Into<Principal> for StablePrincipal {
    fn into(self) -> Principal {
        #[allow(clippy::unwrap_used)] // unwrap expected
        Principal::try_from(self.0.as_slice()).unwrap()
    }
}

/// # Panics
/// Panics if the serialization/deserialization fails.
impl Storable for StablePrincipal {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Borrowed(self.0.as_slice())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        #[allow(clippy::unwrap_used)] // unwrap expected
        Self(Blob::try_from(bytes.as_ref()).unwrap())
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 29,
            is_fixed_size: false,
        };
}

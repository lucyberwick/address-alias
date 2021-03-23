use cosmwasm_std::{CanonicalAddr, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::PrefixedStorage;
use schemars::JsonSchema;
use secret_toolkit::serialization::{Bincode2, Serde};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;

// === STATICS ===
pub static ALIAS_PREFIX: &[u8] = b"alias";
pub static ALIASES_PREFIX: &[u8] = b"aliases";
pub static CONFIG_KEY: &[u8] = b"config";

// === STRUCTS ===
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Alias {
    pub human_address: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub max_alias_size: u16,
}

pub struct AliasStorage<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}

impl<'a, S: Storage> AliasStorage<'a, S> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(ALIAS_PREFIX, storage),
        }
    }

    pub fn get_alias(&mut self, key: &String) -> Option<Alias> {
        self.as_readonly().get(key)
    }

    pub fn remove_alias(&mut self, key: &[u8]) {
        remove(&mut self.storage, &key);
    }

    pub fn set_alias(&mut self, key: &[u8], value: Alias) {
        save(&mut self.storage, &key, &value).ok();
    }

    // private

    fn as_readonly(&self) -> ReadonlyAliasStorageImpl<PrefixedStorage<S>> {
        ReadonlyAliasStorageImpl(&self.storage)
    }
}

struct ReadonlyAliasStorageImpl<'a, S: ReadonlyStorage>(&'a S);
impl<'a, S: ReadonlyStorage> ReadonlyAliasStorageImpl<'a, S> {
    pub fn get(&self, key: &String) -> Option<Alias> {
        let alias: Option<Alias> = may_load(self.0, &key.as_bytes()).ok().unwrap();
        alias
    }
}

pub struct AliasesStorage<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}
impl<'a, S: Storage> AliasesStorage<'a, S> {
    pub fn add_alias(&mut self, key: &CanonicalAddr, alias_string: String) {
        save(
            &mut self.storage,
            &key.as_slice().to_vec(),
            &vec![alias_string],
        )
        .ok();
    }

    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(ALIASES_PREFIX, storage),
        }
    }

    pub fn get_aliases(&mut self, key: &CanonicalAddr) -> Option<Vec<String>> {
        self.as_readonly().get(key)
    }

    // private

    fn as_readonly(&self) -> ReadonlyAliasesStorageImpl<PrefixedStorage<S>> {
        ReadonlyAliasesStorageImpl(&self.storage)
    }
}

struct ReadonlyAliasesStorageImpl<'a, S: ReadonlyStorage>(&'a S);
impl<'a, S: ReadonlyStorage> ReadonlyAliasesStorageImpl<'a, S> {
    pub fn get(&self, key: &CanonicalAddr) -> Option<Vec<String>> {
        let aliases: Option<Vec<String>> = may_load(self.0, &key.as_slice().to_vec()).ok().unwrap();
        aliases
    }
}

// === FUNCTIONS ===

pub fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    Bincode2::deserialize(
        &storage
            .get(key)
            .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
    )
}

pub fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => Bincode2::deserialize(&value).map(Some),
        None => Ok(None),
    }
}

/// Removes an item from storage
///
/// # Arguments
///
/// * `storage` - a mutable reference to the storage this item is in
/// * `key` - a byte slice representing the key that accesses the stored item
pub fn remove<S: Storage>(storage: &mut S, key: &[u8]) {
    storage.remove(key);
}

// Returns StdResult<()> resulting from saving an item to storage
// Arguments:
// storage - a mutable reference to the storage this item should go to
// key - a byte slice representing the key to access the stored item
// value - a reference to the item to store
pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}

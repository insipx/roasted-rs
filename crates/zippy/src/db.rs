//! Storage for Zippy to persist things
use core::marker::PhantomData;
use core::ops::Deref;

use esp_storage::FlashStorage;
use esp_bootloader_esp_idf::partitions::{
    read_partition_table, DataPartitionSubType, PartitionType,
};
use esp_bootloader_esp_idf::partitions::FlashRegion;
use serde::{Serialize, de::DeserializeOwned};
use alloc::collections::BTreeMap;

use crate::error::StorageError;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    inner: [u8; 4]
}

impl Key {
    pub const fn new(v: [u8; 4]) -> Self {
        Self {
            inner: v
        }
    }
}

impl From<[u8; 4]> for Key {
    fn from(v: [u8; 4]) -> Key {
        Key {
            inner: v
        }
    }
}

impl Deref for Key {
    type Target = [u8; 4];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Database<'a, V> {
    db: FlashStorage<'a>,
    _marker: PhantomData<V>,
    cache: BTreeMap<Key, V>

}

pub struct DbRegion<'a, 'd, V> {
    db: FlashRegion<'a, 'd>,
    _marker: PhantomData<V>
}

// fn with_db<V, T, F>(mut flash: FlashStorage, f: F) -> Result<T, StorageError>
// where
//  for<'a, 'd> F: FnOnce(&mut Database<'a, 'd, V>) -> Result<T, StorageError>
// {
//     let mut table_buffer = [0u8; 0xC00];
//     let table = read_partition_table(&mut flash, &mut table_buffer)?;
//
//     let entry = table
//         .find_partition(PartitionType::Data(DataPartitionSubType::Nvs))?
//         .expect("NVS partition missing");
//     let mut flash_region = entry.as_flash_region(&mut flash);
//     let mut db = Database::new(flash_region)?;
//     f(&mut db)
// }

impl<'a, V> Database<'a, V> {
    pub fn new(flash: FlashStorage<'a>) -> Result<Self, StorageError> {
        Ok(Self {
            db: flash,
            cache: Default::default(),
            _marker: PhantomData,
        })
    }
}

impl<'a, 'd, V> DbRegion<'a, 'd, V> {
    pub fn new(flash: FlashRegion<'a, 'd>) -> Result<Self, StorageError> {
        Ok(Self {
            db: flash,
            _marker: PhantomData,
        })
    }
}

impl<'a, V> Database<'a, V> {
    pub fn as_region(&mut self) -> Result<DbRegion<'_, 'a, V>, StorageError> {
        let mut table_buffer = [0u8; 0xC00];
        let table = read_partition_table(&mut self.db, &mut table_buffer)?;

        let entry = table
            .find_partition(PartitionType::Data(DataPartitionSubType::Nvs))?
            .expect("NVS partition missing");
        let mut flash_region = entry.as_flash_region(&mut self.db);
        DbRegion::new(flash_region)
    }
}

impl<V> Database<'_, V> {
    fn put(&mut self, key: Key, value: V) {
        let _ = self.cache.insert(key, value);
    }

    fn get(&self, key: &Key) -> Option<&V> {
        self.cache.get(key)
    }
}

impl<V> Database<'_, V> where V: DeserializeOwned + Serialize {
    fn flush(&mut self) {
        let map = self.cache.
    }
}
    //
    // pub fn write_password(pw: &[u8]) -> Result<(), StorageError> {
    //     self.
    // }

struct Keys;
impl Keys {
    /// WiFi SSID
    pub const SSID: Key = Key::new(*b"SSID");
    /// WiFi Password
    pub const WIPW: Key = Key::new(*b"WIPW");
}


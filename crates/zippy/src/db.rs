//! Storage for Zippy to persist things
use alloc::collections::BTreeMap;
use core::{marker::PhantomData, ops::Deref};

use esp_bootloader_esp_idf::partitions::{
    DataPartitionSubType, FlashRegion, PartitionType, read_partition_table,
};
use esp_storage::FlashStorage;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::StorageError;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Key {
    inner: [u8; 4],
}

impl Key {
    pub const fn new(v: [u8; 4]) -> Self {
        Self { inner: v }
    }
}

impl From<[u8; 4]> for Key {
    fn from(v: [u8; 4]) -> Key {
        Key { inner: v }
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
    cache: BTreeMap<Key, V>,
}

pub struct DbRegion<'a, 'd, V> {
    db: FlashRegion<'a, 'd>,
    _marker: PhantomData<V>,
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
        Ok(Self { db: flash, cache: Default::default(), _marker: PhantomData })
    }
}

impl<'a, 'd, V> DbRegion<'a, 'd, V> {
    pub fn new(flash: FlashRegion<'a, 'd>) -> Result<Self, StorageError> {
        Ok(Self { db: flash, _marker: PhantomData })
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
    pub fn put(&mut self, key: Key, value: V) {
        let _ = self.cache.insert(key, value);
    }

    pub fn get(&self, key: &Key) -> Option<&V> {
        self.cache.get(key)
    }
}

impl<V> Database<'_, V>
where
    V: DeserializeOwned + Serialize,
{
    /// Flush the DB, persisting all values.
    pub fn flush(&mut self) -> Result<(), StorageError> {
        // let mut buf = [0u8; 1024];
        // let bytes = postcard::to_slice(&self.cache, &mut buf)?;
        // self.db.write_encrypted(0, &buf);
        Ok(())
    }
}

struct Keys;
impl Keys {
    /// WiFi SSID
    pub const SSID: Key = Key::new(*b"SSID");
    /// WiFi Password
    pub const WIPW: Key = Key::new(*b"WIPW");
}

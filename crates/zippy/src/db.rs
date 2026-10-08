//! Storage for Zippy to persist things
use alloc::collections::BTreeMap;
use core::{marker::PhantomData, ops::Deref};

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{AeadInOut, Key, KeyInit, arrayvec::ArrayVec},
};
use crypto_common::Generate;
use esp_bootloader_esp_idf::partitions::{
    DataPartitionSubType, FlashRegion, PartitionType, read_partition_table,
};
use esp_hal::rng::{Trng, TrngSource};
use esp_storage::FlashStorage;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::StorageError;

const CURRENT_VERSION: u8 = 1;
const AEAD: &[u8; 16] = b"roasted/cred/v01";

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DbKey {
    inner: [u8; 4],
}

impl DbKey {
    pub const fn new(v: [u8; 4]) -> Self {
        Self { inner: v }
    }
}

impl From<[u8; 4]> for DbKey {
    fn from(v: [u8; 4]) -> DbKey {
        DbKey { inner: v }
    }
}

impl Deref for DbKey {
    type Target = [u8; 4];
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Database<'a, V> {
    db: FlashStorage<'a>,
    _marker: PhantomData<V>,
    cache: BTreeMap<DbKey, V>,
    entropy: TrngSource<'a>,
    rng: Trng,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry<'a> {
    version: u8,
    nonce: [u8; 24],
    ciphertext: &'a [u8],
    auth_tag: [u8; 16],
}

impl<'a, V> Database<'a, V> {
    pub fn new(flash: FlashStorage<'a>, entropy: TrngSource<'a>) -> Result<Self, StorageError> {
        let rng = Trng::try_new()?;
        Ok(Self { db: flash, cache: Default::default(), rng, entropy, _marker: PhantomData })
    }
}

impl<'a, V> Database<'a, V> {
    pub fn as_region(&mut self) -> Result<FlashRegion<'_, 'a>, StorageError> {
        let mut table_buffer = [0u8; 0xC00];
        let table = read_partition_table(&mut self.db, &mut table_buffer)?;
        let db = table.iter().find(|entry| {
            entry.label_as_str() == "secrets"
                && entry.partition_type() == PartitionType::Data(DataPartitionSubType::Undefined)
        });
        let Some(db) = db else {
            return Err(StorageError::PartitionDoesNotExist);
        };
        Ok(db.as_flash_region(&mut self.db))
    }

    /// Generate a new nonce
    fn nonce(&mut self) -> XNonce {
        XNonce::generate_from_rng(&mut self.rng)
    }
}

impl<V> Database<'_, V> {
    pub fn put(&mut self, key: DbKey, value: V) {
        let _ = self.cache.insert(key, value);
    }

    pub fn get(&self, key: &DbKey) -> Option<&V> {
        self.cache.get(key)
    }
}

impl<V> Database<'_, V>
where
    V: DeserializeOwned + Serialize,
{
    /// Flush the DB, persisting all values.
    pub fn flush(&mut self) -> Result<(), StorageError> {
        let mut tree: ArrayVec<u8, 1024> = ArrayVec::new();
        postcard::to_slice(&self.cache, &mut tree.as_mut_slice())?;
        let nonce = self.nonce();
        let key = Key::<XChaCha20Poly1305>::generate_from_rng(&mut self.rng);
        let cipher = XChaCha20Poly1305::new(&key);
        cipher.encrypt_in_place(&nonce, b"", &mut tree)?;
        let envelope_bytes = Entry {
            version: CURRENT_VERSION,
            nonce: nonce.0,
            ciphertext: tree.as_slice(),
            auth_tag: *AEAD,
        };
        let mut envelope = [0u8; 1024];
        postcard::to_slice(&envelope_bytes, &mut envelope)?;

        let mut region = self.as_region()?;
        region.write(0, &mut envelope)?;
        Ok(())
    }
}

pub struct Keys;
impl Keys {
    /// WiFi SSID
    pub const SSID: DbKey = DbKey::new(*b"SSID");
    /// WiFi Password
    pub const WIPW: DbKey = DbKey::new(*b"WIPW");
    /// Misc
    pub const MISC: DbKey = DbKey::new(*b"MISC");
}

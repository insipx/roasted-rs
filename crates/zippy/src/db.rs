//! Storage for Zippy to persist things
use alloc::{collections::BTreeMap, format};
use core::{marker::PhantomData, ops::Deref};

use anyhow::{Context, Result, bail};
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
use postcard::experimental::serialized_size;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::StorageError;

const CURRENT_VERSION: u8 = 1;
const AEAD: &[u8; 16] = b"roasted/cred/v01";
const MEM_SIZE: usize = 1536;
const TREE_SIZE: usize = 1024;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DbKey {
    inner: [u8; 4],
}

impl core::fmt::Display for DbKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = core::primitive::str::from_utf8(&self.inner).expect("key values are static");
        write!(f, "{}", s)
    }
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

// need to always load tree from flash on creation
pub struct Database<'a, V> {
    db: FlashStorage<'a>,
    _marker: PhantomData<V>,
    cache: BTreeMap<DbKey, V>,
    entropy: TrngSource<'a>,
    rng: Trng,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    version: u8,
    nonce: [u8; 24],
    ciphertext: ArrayVec<u8, TREE_SIZE>,
    auth_tag: [u8; 16],
}

impl<'a, V> Database<'a, V> {
    pub fn as_region(&mut self) -> Result<FlashRegion<'_, 'a>> {
        let mut table_buffer = [0u8; 0xC00];
        let table = read_partition_table(&mut self.db, &mut table_buffer)?;
        let db = table.iter().find(|entry| {
            entry.label_as_str() == "secrets"
                && entry.partition_type() == PartitionType::Data(DataPartitionSubType::Undefined)
        });
        let Some(db) = db else {
            bail!("The encrypted data partition does not exist. try re-flashing.");
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

impl<'a, V> Database<'a, V>
where
    V: DeserializeOwned + Serialize + AsRef<[u8]>,
{
    pub fn new(flash: FlashStorage<'a>, entropy: TrngSource<'a>) -> Result<Self> {
        let rng = Trng::try_new().map_err(|_| StorageError::Rng)?;
        let mut this =
            Self { db: flash, cache: Default::default(), rng, entropy, _marker: PhantomData };
        if let Err(e) = this.refresh() {
            esp_println::println!("DB could not read persisted data, discarding {e}");
        }
        for (k, v) in this.cache.iter() {
            if *k == Keys::WIPW {
                esp_println::println!("{k}: ***");
            } else {
                // for now we only store string pairs. this will probably break. not worth
                // expanding value types yet, we only need wifi pass.
                let s = core::primitive::str::from_utf8(v.as_ref())
                    .expect("whoops we dont handle non-UTF8 key values!");
                esp_println::println!("{}:{}", k, s);
            }
        }
        Ok(this)
    }

    #[cfg(feature = "debug")]
    fn key(&mut self) -> Key<XChaCha20Poly1305> {
        let mut key = [0u8; 32];
        hex::decode_to_slice(
            "e089b4470197061b0dd9e8bd3ae94967299d385276028d1016687f07b6f7b5ed",
            &mut key,
        )
        .expect("static decode works");
        key.into()
    }

    #[cfg(not(feature = "debug"))]
    fn key(&mut self) -> Key<XChaCha20Poly1305> {
        Key::<XChaCha20Poly1305>::generate_from_rng(&mut self.rng)
    }

    fn refresh(&mut self) -> Result<()> {
        let mut region = self.as_region()?;
        let mut buf = [0u8; MEM_SIZE];
        region.read(0, &mut buf)?;
        let key = self.key();
        let mut entry: Entry = postcard::from_bytes(&buf)
            .with_context(|| format!("failed reading persisted memory"))?;
        let cipher = XChaCha20Poly1305::new(&key);
        cipher.decrypt_in_place(&entry.nonce.into(), &entry.auth_tag, &mut entry.ciphertext)?;
        let tree: BTreeMap<DbKey, V> = postcard::from_bytes(&entry.ciphertext)?;
        let _ = core::mem::replace(&mut self.cache, tree);
        Ok(())
    }

    /// Flush the DB, persisting all values.
    pub fn flush(&mut self) -> Result<()> {
        let tree: ArrayVec<u8, TREE_SIZE> = ArrayVec::new();
        let mut tree = postcard::to_extend(&self.cache, tree).with_context(|| {
            format!(
                "failed to persist memory (size {}) to tree slice",
                serialized_size(&self.cache).expect("hope it works")
            )
        })?;
        let nonce = self.nonce();
        let key = self.key();
        let cipher = XChaCha20Poly1305::new(&key);
        cipher.encrypt_in_place(&nonce, AEAD, &mut tree)?;
        let envelope_bytes =
            Entry { version: CURRENT_VERSION, nonce: nonce.0, ciphertext: tree, auth_tag: *AEAD };
        let mut envelope = [0u8; MEM_SIZE];
        postcard::to_slice(&envelope_bytes, &mut envelope).with_context(|| {
            format!(
                "failed to persist memory size({}) to envelope",
                serialized_size(&envelope_bytes).expect("hope it works")
            )
        })?;

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

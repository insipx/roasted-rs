//! Storage for Zippy to persist things
use alloc::{collections::BTreeMap, format, sync::Arc};
use core::{marker::PhantomData, ops::Deref};

use anyhow::{Context, Result, anyhow, bail};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{AeadInOut, Key, KeyInit, arrayvec::ArrayVec},
};
use crypto_common::Generate;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use esp_bootloader_esp_idf::partitions::{
    DataPartitionSubType, FlashRegion, PartitionType, read_partition_table,
};
use esp_hal::{
    hmac::{Hmac, HmacPurpose, KeyId},
    rng::{Trng, TrngSource},
};
use esp_storage::FlashStorage;
use postcard::experimental::serialized_size;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::StorageError;

const CURRENT_VERSION: u8 = 1;
const AEAD: &[u8; 16] = b"roasted/cred/v01";
const MEM_SIZE: usize = 1536;
const TREE_SIZE: usize = 1024;
const HMAC_LABEL: &[u8; 29] = b"roasted/zippy/database-key/v1";
const HEADER_SIZE: usize = 8;
const DB_HEADER: &[u8; HEADER_SIZE] = b"ZIPPY_DB";
const DB_START: u32 = 8;

type Lock<T> = Arc<RwLock<CriticalSectionRawMutex, BTreeMap<DbKey, T>>>;

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

/// Read-only db
pub struct DbReader<V> {
    inner: Lock<V>,
}

impl<V> DbReader<V> {
    pub async fn get(&self, key: &DbKey) -> Option<&V> {
        let read = self.inner.read().await;
        read.get(key)
    }
}

// TODO: Protect against power faults. if a power fault occurs
// mid write we will need to re-provision.
// for now the worst that happens is re provisioning.
// the db is ok for what we need for now.
// should consider some kind of A/B version update strategy.
// or `sequential-storage`` i guess.
pub struct Database<'a, V> {
    db: FlashStorage<'a>,
    _marker: PhantomData<V>,
    cache: Lock<V>,
    rng: Trng,
    entropy: TrngSource<'a>,
    key: Key<XChaCha20Poly1305>,
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

    fn reader(&self) -> DbReader<V> {
        DbReader { inner: self.cache.clone() }
    }
}

pub struct EncryptionKey {
    inner: Key<XChaCha20Poly1305>,
}

impl EncryptionKey {
    /// Use the hardcoded debug key (for debugging only)
    #[cfg(feature = "debug")]
    pub fn debug() -> Self {
        let mut key = [0u8; 32];
        hex::decode_to_slice(
            "e089b4470197061b0dd9e8bd3ae94967299d385276028d1016687f07b6f7b5ed",
            &mut key,
        )
        .expect("static decode works");

        Self { inner: key.into() }
    }

    /// Use an hmac key derived from BLOCK0 of eFuse
    pub fn hmac(hmac: esp_hal::peripherals::HMAC) -> Result<Self> {
        let mut hmac = Hmac::new(hmac);
        hmac.init();
        nb::block!(hmac.configure(HmacPurpose::ToUser, KeyId::Key0))
            .map_err(|_| anyhow!("configuring hmac block failed"))?;

        let mut remaining: &[u8] = &HMAC_LABEL.clone();
        while !remaining.is_empty() {
            remaining = nb::block!(hmac.update(remaining))?;
        }

        let mut db_key = [0u8; 32];
        nb::block!(hmac.finalize(&mut db_key))?;
        Ok(Self { inner: db_key.into() })
    }
}

impl<V> Database<'_, V> {
    pub async fn put(&mut self, key: DbKey, value: V) {
        let mut cache = self.cache.write().await;
        let _ = cache.insert(key, value);
    }

    pub async fn get(&self, key: &DbKey) -> Option<&V> {
        let cache = self.cache.read().await;
        cache.get(key)
    }
}

impl<'a, V> Database<'a, V>
where
    V: DeserializeOwned + Serialize + AsRef<[u8]>,
{
    pub fn new(
        flash: FlashStorage<'a>,
        entropy: TrngSource<'a>,
        key: EncryptionKey,
    ) -> Result<Self> {
        let rng = Trng::try_new().map_err(|_| StorageError::Rng)?;
        let mut this = Self {
            db: flash,
            cache: Default::default(),
            rng,
            entropy,
            _marker: PhantomData,
            key: key.inner,
        };

        // try to read the header. if all bytes are 0xFF db is new
        // if header doesn't match, corrupted b
        // otherwise try to refresh values.
        let mut buf = [0u8; HEADER_SIZE];
        let mut region = this.as_region()?;
        region.read(0, &mut buf)?;
        if let Ok(header) = core::primitive::str::from_utf8(&buf) {
            esp_println::println!("db header: `{header}`");
        }
        if buf.iter().all(|&byte| byte == 0xFF) {
            esp_println::println!("brand new database");
            region.write(0, DB_HEADER)?;
            this.cache.clear();
        } else if buf != *DB_HEADER {
            // not the best but idc for now.
            esp_println::println!("unknown/corrupted db. discarding and overwriting");
            region.write(0, DB_HEADER)?;
            this.cache.clear();
            let _ = this.flush();
        } else {
            this.refresh()?;
        }
        for (k, v) in this.cache.iter() {
            if *k == Keys::WIPW {
                esp_println::println!("{k}: ***");
            } else {
                // for now we only store string pairs. this will probably break. not worth
                // expanding value types yet, we only need wifi pass.
                let s = core::primitive::str::from_utf8(v.as_ref())
                    .expect("whoops we do not handle non-UTF8 key values!");
                esp_println::println!("{} : {}", k, s);
            }
        }
        Ok(this)
    }

    #[cfg(not(feature = "debug"))]
    fn key(&mut self) -> Key<XChaCha20Poly1305> {
        Key::<XChaCha20Poly1305>::generate_from_rng(&mut self.rng)
    }

    fn refresh(&mut self) -> Result<()> {
        let mut region = self.as_region()?;
        let mut buf = [0u8; MEM_SIZE];
        region.read(DB_START, &mut buf)?;
        let mut entry: Entry = postcard::from_bytes(&buf)
            .with_context(|| format!("failed reading persisted memory"))?;
        let cipher = XChaCha20Poly1305::new(&self.key);
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
        let cipher = XChaCha20Poly1305::new(&self.key);
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
        region.write(DB_START, &mut envelope)?;
        Ok(())
    }
}

pub struct Keys;
impl Keys {
    /// WiFi SSID
    pub const SSID: DbKey = DbKey::new(*b"SSID");
    /// WiFi Password
    pub const WIPW: DbKey = DbKey::new(*b"WIPW");
    /// URL of the roasted-rs ws daemon
    pub const DURL: DbKey = DbKey::new(*b"DURL");
    /// Misc
    pub const MISC: DbKey = DbKey::new(*b"MISC");
}

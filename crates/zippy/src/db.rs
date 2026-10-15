//! Storage for Zippy to persist things
use alloc::{collections::BTreeMap, format, sync::Arc};
use core::{marker::PhantomData, ops::Deref};

use anyhow::{Context, Result, bail};
use arrayvec::ArrayVec;
use chacha20poly1305::{AeadInOut, XChaCha20Poly1305, aead::Key};
use crypto_common::KeyInit;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, rwlock::RwLock};
use esp_bootloader_esp_idf::partitions::{
    DataPartitionSubType, FlashRegion, PartitionType, read_partition_table,
};
use esp_hal::rng::Trng;
use esp_storage::FlashStorage;
use postcard::experimental::serialized_size;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::StorageError;

mod encryption;
mod read;
pub use encryption::*;
pub use read::*;

use crate::configuration::db::*;

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

    pub fn reader(&self) -> DbReader<V> {
        DbReader { inner: self.cache.clone() }
    }
}

impl<V> Database<'_, V> {
    pub async fn put(&mut self, key: DbKey, value: V) {
        let mut cache = self.cache.write().await;
        let _ = cache.insert(key, value);
    }

    pub async fn read(&self) -> DbReadView<'_, V> {
        DbReadView { guard: self.cache.read().await }
    }
}

impl<'a, V> Database<'a, V>
where
    V: DeserializeOwned + Serialize + AsRef<[u8]>,
{
    pub fn new(flash: FlashStorage<'a>, key: EncryptionKey) -> Result<Self> {
        let rng = Trng::try_new().map_err(|_| StorageError::Rng)?;
        let mut this = Self {
            db: flash,
            cache: Default::default(),
            rng,
            _marker: PhantomData,
            key: key.inner,
        };
        let mut cache: BTreeMap<DbKey, V> = BTreeMap::default();

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
            cache.clear();
        } else if buf != *DB_HEADER {
            // not the best but idc for now.
            esp_println::println!("unknown/corrupted db. discarding and overwriting");
            region.write(0, DB_HEADER)?;
            cache.clear();
            let _ = this.flush();
        } else {
            cache = this.refresh()?;
        }
        for (k, v) in cache.iter() {
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
        this.cache = Arc::new(RwLock::new(cache));
        Ok(this)
    }

    fn refresh(&mut self) -> Result<BTreeMap<DbKey, V>> {
        let mut region = self.as_region()?;
        let mut buf = [0u8; MEM_SIZE];
        region.read(DB_START, &mut buf)?;
        let mut entry: Entry = postcard::from_bytes(&buf)
            .with_context(|| format!("failed reading persisted memory"))?;
        let cipher = XChaCha20Poly1305::new(&self.key);
        cipher.decrypt_in_place(&entry.nonce.into(), &entry.auth_tag, &mut entry.ciphertext)?;
        let tree: BTreeMap<DbKey, V> = postcard::from_bytes(&entry.ciphertext)?;
        Ok(tree)
    }

    /// Flush the DB, persisting all values.
    pub async fn flush(&mut self) -> Result<()> {
        let tree: ArrayVec<u8, TREE_SIZE> = ArrayVec::new();
        let mut tree = {
            let cache = self.cache.read().await;
            postcard::to_extend(&*cache, tree).with_context(|| {
                format!(
                    "failed to persist memory (size {}) to tree slice",
                    serialized_size(&*cache).expect("hope it works")
                )
            })
        }?;
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

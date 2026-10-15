use anyhow::{Result, anyhow};
use chacha20poly1305::{XChaCha20Poly1305, XNonce, aead::Key};
use crypto_common::Generate;
use esp_hal::hmac::{Hmac, HmacPurpose, KeyId};

use super::Database;
use crate::configuration::db::HMAC_LABEL;

pub struct EncryptionKey {
    pub(super) inner: Key<XChaCha20Poly1305>,
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

impl<'a, V> Database<'a, V> {
    /// Generate a new nonce
    pub(super) fn nonce(&mut self) -> XNonce {
        XNonce::generate_from_rng(&mut self.rng)
    }

    #[cfg(not(feature = "debug"))]
    fn key(&mut self) -> Key<XChaCha20Poly1305> {
        Key::<XChaCha20Poly1305>::generate_from_rng(&mut self.rng)
    }
}

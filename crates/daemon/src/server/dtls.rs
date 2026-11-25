use std::{ffi::CString, path::PathBuf};

use color_eyre::eyre::{Result, WrapErr};

/// DTLS Key Pair
#[derive(Clone, zeroize::Zeroize)]
pub struct DtlsKeys {
    /// TLS Private Key
    pub key: CString,
    /// TLS Public Key Certificate Chain
    pub cert: CString,
}

impl DtlsKeys {
    pub fn new() -> Result<Self> {
        let dtls_key: PathBuf = PathBuf::from(
            std::env::var("DTLS_KEY_PATH").wrap_err("dtls key path must exist in environment")?,
        );
        let dtls_crt: PathBuf = PathBuf::from(
            std::env::var("DTLS_CRT_PATH").wrap_err("dtls cert path must exist in environment")?,
        );
        let key: String =
            std::fs::read_to_string(dtls_key).wrap_err("failed reading dtls private key")?;
        let cert: String = std::fs::read_to_string(dtls_crt)
            .wrap_err("failed reading dtls public certificate chain")?;
        Ok(Self { cert: CString::new(cert)?, key: CString::new(key)? })
    }
}

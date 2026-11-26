use std::mem::MaybeUninit;

use color_eyre::eyre::{Result, bail};
use mbedtls_rs_sys::{mbedtls_psa_external_random_context_t, psa_crypto_init, psa_status_t};

const PSA_SUCCESS: psa_status_t = 0;
const PSA_ERROR_INSUFFICIENT_ENTROPY: psa_status_t = -148;

pub fn init() -> Result<()> {
    let status = unsafe { psa_crypto_init() };
    if status != PSA_SUCCESS {
        bail!("failed to initialize Mbed TLS PSA Crypto: {status}");
    }
    Ok(())
}

// provide psa rng implementation to mbedtls
#[unsafe(no_mangle)]
unsafe extern "C" fn mbedtls_psa_external_get_random(
    _context: *mut mbedtls_psa_external_random_context_t,
    output: *mut u8,
    output_size: usize,
    output_length: *mut usize,
) -> psa_status_t {
    unsafe { output_length.write(0) };
    if output_size == 0 {
        return PSA_SUCCESS;
    }

    let bytes =
        unsafe { std::slice::from_raw_parts_mut(output.cast::<MaybeUninit<u8>>(), output_size) };
    if getrandom::fill_uninit(bytes).is_err() {
        return PSA_ERROR_INSUFFICIENT_ENTROPY;
    }

    unsafe { output_length.write(output_size) };
    PSA_SUCCESS
}

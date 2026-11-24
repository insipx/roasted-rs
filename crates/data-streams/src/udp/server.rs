use core::{ffi::CStr, net::SocketAddr, ptr};

use mbedtls_rs_sys::{
    MBEDTLS_SSL_IS_SERVER, MBEDTLS_SSL_VERIFY_NONE, mbedtls_pk_check_pair, mbedtls_pk_context,
    mbedtls_pk_free, mbedtls_pk_init, mbedtls_pk_parse_key, mbedtls_ssl_conf_authmode,
    mbedtls_ssl_conf_dtls_cookies, mbedtls_ssl_conf_own_cert, mbedtls_ssl_cookie_check,
    mbedtls_ssl_cookie_ctx, mbedtls_ssl_cookie_free, mbedtls_ssl_cookie_init,
    mbedtls_ssl_cookie_setup, mbedtls_ssl_cookie_write, mbedtls_ssl_session_reset,
    mbedtls_ssl_set_client_transport_id, mbedtls_x509_crt, mbedtls_x509_crt_free,
    mbedtls_x509_crt_init, mbedtls_x509_crt_parse,
};

use super::{
    DtlsSetupError, MaybeUninit, TryCryptoRng, UdpDtlsSocket, UdpSocket, c_int, check_setup,
    rng_callback,
};

pub(super) struct ServerState {
    chain: mbedtls_x509_crt,
    key: mbedtls_pk_context,
    cookies: mbedtls_ssl_cookie_ctx,
    peer_id: Vec<u8>,
}

impl ServerState {
    fn new(peer: SocketAddr) -> Self {
        let mut chain = MaybeUninit::uninit();
        let mut key = MaybeUninit::uninit();
        let mut cookies = MaybeUninit::uninit();
        unsafe {
            mbedtls_x509_crt_init(chain.as_mut_ptr());
            mbedtls_pk_init(key.as_mut_ptr());
            mbedtls_ssl_cookie_init(cookies.as_mut_ptr());
            Self {
                chain: chain.assume_init(),
                key: key.assume_init(),
                cookies: cookies.assume_init(),
                peer_id: peer.to_string().into_bytes(),
            }
        }
    }
}

impl Drop for ServerState {
    fn drop(&mut self) {
        unsafe {
            mbedtls_ssl_cookie_free(&mut self.cookies);
            mbedtls_pk_free(&mut self.key);
            mbedtls_x509_crt_free(&mut self.chain);
        }
    }
}

impl<'net> UdpDtlsSocket<'net> {
    pub fn new_server(
        socket: Box<dyn UdpSocket + 'net>,
        rng: impl TryCryptoRng + 'net,
        peer: SocketAddr,
        certificate_chain: &CStr,
        private_key: &CStr,
    ) -> Result<Self, DtlsSetupError> {
        let mut session =
            Self::new_with_endpoint(socket, rng, MBEDTLS_SSL_IS_SERVER as c_int, |session| {
                let rng = session.rng_ptr();
                let server = session.server.insert(Box::new(ServerState::new(peer)));
                check_setup("parse server certificate chain", unsafe {
                    mbedtls_x509_crt_parse(
                        &mut server.chain,
                        certificate_chain.as_ptr().cast(),
                        certificate_chain.to_bytes_with_nul().len(),
                    )
                })?;
                check_setup("parse server private key", unsafe {
                    mbedtls_pk_parse_key(
                        &mut server.key,
                        private_key.as_ptr().cast(),
                        private_key.to_bytes_with_nul().len(),
                        ptr::null(),
                        0,
                        Some(rng_callback),
                        rng,
                    )
                })?;
                check_setup("check server key pair", unsafe {
                    mbedtls_pk_check_pair(&server.chain.pk, &server.key, Some(rng_callback), rng)
                })?;
                check_setup("configure server certificate", unsafe {
                    mbedtls_ssl_conf_own_cert(
                        session.ssl_config.as_mut(),
                        &mut server.chain,
                        &mut server.key,
                    )
                })?;
                check_setup("initialize DTLS cookies", unsafe {
                    mbedtls_ssl_cookie_setup(&mut server.cookies, Some(rng_callback), rng)
                })?;
                unsafe {
                    mbedtls_ssl_conf_authmode(
                        session.ssl_config.as_mut(),
                        MBEDTLS_SSL_VERIFY_NONE as c_int,
                    );
                    mbedtls_ssl_conf_dtls_cookies(
                        session.ssl_config.as_mut(),
                        Some(mbedtls_ssl_cookie_write),
                        Some(mbedtls_ssl_cookie_check),
                        ptr::from_mut(&mut server.cookies).cast(),
                    );
                }
                Ok(())
            })?;
        check_setup("set client transport ID", session.set_client_transport_id())?;
        Ok(session)
    }

    fn set_client_transport_id(&mut self) -> c_int {
        let Some(server) = &self.server else {
            return mbedtls_rs_sys::MBEDTLS_ERR_SSL_BAD_INPUT_DATA;
        };
        unsafe {
            mbedtls_ssl_set_client_transport_id(
                self.ssl_context.as_mut(),
                server.peer_id.as_ptr(),
                server.peer_id.len(),
            )
        }
    }

    pub(super) fn reset_server_session(&mut self) -> c_int {
        let code = unsafe { mbedtls_ssl_session_reset(self.ssl_context.as_mut()) };
        if code != 0 {
            return code;
        }
        self.set_client_transport_id()
    }
}

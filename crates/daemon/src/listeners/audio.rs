use pin_project_lite::pin_project;

pub struct AudioListener {
    udp_socket: UdpDtlsSocket<'static>,
}

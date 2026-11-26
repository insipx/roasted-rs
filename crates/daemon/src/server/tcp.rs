//! The TCP Server
//! Spawns drivers upon connections with Zippy clients

use std::path::PathBuf;

use color_eyre::Result;
use tokio::net::{TcpListener, TcpSocket};
use url::Url;

use crate::server::{driver::RoastedDriver, dtls::DtlsKeys};

pub async fn server(port: u16, gaggimate: Url, db: PathBuf, dtls: DtlsKeys) -> Result<()> {
    let socket = TcpSocket::new_v4()?;
    socket.set_keepalive(true)?;
    socket.bind(format!("0.0.0.0:{port}").parse()?)?;
    let listener = socket.listen(1024)?;
    tracing::info!("listening on 0.0.0.0:{port}");
    loop {
        let (socket, addr) = listener.accept().await?;
        tracing::info!("accepted connection from {addr}");
        let gm = gaggimate.clone();
        let db = db.clone();
        let driver = RoastedDriver::new(gm, socket, db, dtls.clone());
        tokio::task::spawn_local(async move {
            if let Err(error) = driver.run().await {
                tracing::error!(%addr, ?error, "driver failed");
            }
        });
    }
}

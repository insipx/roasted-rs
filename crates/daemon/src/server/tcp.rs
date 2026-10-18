//! The TCP Server
//! Spawns drivers upon connections with Zippy clients

use std::path::PathBuf;

use color_eyre::Result;
use futures::FutureExt;
use tokio::net::TcpListener;
use url::Url;

use crate::server::driver::RoastedDriver;

pub async fn server(port: u16, gaggimate: Url, db: PathBuf) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    loop {
        let (socket, _) = listener.accept().await?;
        let gm = gaggimate.clone();
        let db = db.clone();
        let driver = RoastedDriver::new(gm, socket, db);
        let _handle = tokio::task::spawn_local(driver.run()).map(|jf| match jf {
            Err(e) => tracing::error!("tokio driver task failed execution {}", e),
            Ok(Err(e)) => tracing::error!("driver failed {}", e),
            Ok(Ok(_)) => (),
        });
    }
}

//! Drives the execution of the Roasted event loop.
//!  - Starts the Gaggimate Listener
//!  - Asks for audio feedback from ESP32/begins conversation sequences
//!  - Handles communication with Ollama local LLM
//!  - Spawned by server upon connection with Zippy

use std::path::PathBuf;

use color_eyre::eyre::Result;
use futures::TryStreamExt;
use roasted_db::Db;
use tokio::net::TcpStream;
use url::Url;

use crate::listeners::gaggimate::GaggimateListener;

pub struct RoastedDriver {
    gaggimate: Url,
    socket: TcpStream,
    db: PathBuf,
}

impl RoastedDriver {
    pub fn new(gaggimate: Url, socket: TcpStream, db: PathBuf) -> Self {
        Self { gaggimate, socket, db }
    }

    /// Run the driver
    pub async fn run(self) -> Result<()> {
        run(self).await
    }
}

/// Orchestrate the roasted daemon.
/// Spawned when a zippy connects and closes when it disconnects.
async fn run(driver: RoastedDriver) -> Result<()> {
    let RoastedDriver { gaggimate, socket, db } = driver;

    let mut listener = GaggimateListener::connect(&gaggimate).await?;
    let mut db = Db::connect(&db)?;

    while let Some(frames) = listener.try_next().await? {
        db.record_frames(&frames)?;
        // derive features
        // initiate LLM facts recital of first shot profiles
        // Ask for shot taste
        // give judgement
        // conversation sequence
        // end
        //
    }
    Ok(())
}

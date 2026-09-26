//! Drives the execution of the Roasted event loop.
//!  - Starts the Gaggimate Listener
//!  - Asks for audio feedback from ESP32/begins conversation sequences
//!  - Handles communication with Ollama local LLM
//!  - Spawned by server upon connection with Zippy

use std::path::PathBuf;

use color_eyre::eyre::Result;
use futures::TryStreamExt;
use roasted_db::Db;
use url::Url;
use warp::filters::ws::WebSocket;

use crate::listeners::gaggimate::GaggimateListener;

pub struct RoastedDriver {
    pub gaggimate: Url,
    pub ws: WebSocket,
    pub db: PathBuf,
}

impl RoastedDriver {
    pub fn new(gaggimate: Url, ws: WebSocket, db: PathBuf) -> Self {
        Self { gaggimate, ws, db }
    }

    /// Run the driver
    pub async fn run(self) -> Result<()> {
        run(self).await
    }
}

/// Orchestrate the roasted daemon.
/// Spawned when a zippy connects and closes when it disconnects.
async fn run(driver: RoastedDriver) -> Result<()> {
    let RoastedDriver { gaggimate, ws, db } = driver;

    let mut listener = GaggimateListener::connect(&gaggimate).await?;
    let mut db = Db::connect(&db)?;

    while let Some(frames) = listener.try_next().await? {
        db.record_frames(&frames)?;
        // derive features
        // initiate LLM facts recital of first shot profiles
        // Ask for shot taste
        // give judgement
        // covnersation sequence
        // end
        //
    }
    Ok(())
}

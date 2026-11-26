//! Drives the execution of the Roasted event loop.
//!  - Starts the Gaggimate Listener
//!  - Asks for audio feedback from ESP32/begins conversation sequences
//!  - Handles communication with Ollama local LLM
//!  - Spawned by server upon connection with Zippy

use std::{net::SocketAddr, path::PathBuf};

use color_eyre::eyre::Result;
use futures::TryStreamExt;
use roasted_data_streams::zippy::ResponseStream;
use roasted_db::Db;
use tokio::net::{TcpStream, tcp::OwnedWriteHalf};
use tokio_tungstenite::tungstenite::handshake::client::Response;
use url::Url;

use crate::{
    listeners::{audio::AudioListener, gaggimate::GaggimateListener},
    server::dtls::DtlsKeys,
};

pub struct RoastedDriver {
    gaggimate: Url,
    responses: ResponseStream,
    db: PathBuf,
    dtls: DtlsKeys,
    tcp_write: OwnedWriteHalf,
}

impl RoastedDriver {
    pub fn new(gaggimate: Url, socket: TcpStream, db: PathBuf, dtls: DtlsKeys) -> Self {
        let (read, write) = socket.into_split();
        Self { gaggimate, responses: ResponseStream::new_tokio(read), tcp_write: write, db, dtls }
    }

    /// Run the driver
    pub async fn run(self) -> Result<()> {
        run(self).await
    }
}

/// Orchestrate the roasted daemon.
/// Spawned when a zippy connects and closes when it disconnects.
async fn run(driver: RoastedDriver) -> Result<()> {
    let RoastedDriver { gaggimate, mut responses, db, dtls, .. } = driver;
    // TODO: Add a timeout/retry loop
    let mut listener = GaggimateListener::connect(&gaggimate).await?;
    tracing::info!("connected to gaggimate");
    let audio = AudioListener::connect("0.0.0.0:54444".parse()?, &dtls.cert, &dtls.key);

    let mut audio = tokio::select! {
        udp = audio => {
            udp
        },
        // in case we need to restart con b/c of a tcp reconnect
        response = responses.try_next() => {
            let Some(response) = response? else {
                return Ok(());
            };
            tracing::info!("got zippy response {:?}", response);
            panic!("Received zippy response before udp conn");
        }
    }?;

    tracing::info!(port = 54444, "connected to UDP socket");
    let mut db = Db::connect(&db)?;
    loop {
        tokio::select! {
            frames = listener.try_next() => {
                let Some(frames) = frames? else {
                    break;
                };
                db.record_frames(&frames)?;
                // derive features
                // initiate LLM facts recital of first shot profiles
                // Ask for shot taste
                // give judgement
                // conversation sequence
                // end
            },
            audio = audio.try_next() => {
                let Some((header, pcm)) = audio? else {
                    break;
                };
                tracing::debug!(sequence = header.sequence, len = pcm.len(), "received audio");
            },
            response = responses.try_next() => {
                let Some(response) = response? else {
                    break;
                };
                tracing::info!("got zippy response {:?}", response);
            }
        }
    }
    Ok(())
}

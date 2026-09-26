use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use color_eyre::eyre::Result;
use futures::{Stream, TryStream, TryStreamExt};
use pin_project_lite::pin_project;
use roasted_types::{
    daemon::{GaggimateState, Merge},
    ws::gaggimate::Message as GmMessage,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message as WsMessage, error::Error as TungError},
};
use url::Url;
use uuid::Uuid;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pin_project! {
    /// Listens to Gaggimate and records each shot under a unique ID.
    /// On shot completion, returns the UUID of the shot recorded in the database.
    pub struct GaggimateListener {
        #[pin]
        inner: GaggimateStream<WsStream>,
        current_state: GaggimateState,
        is_pulling_shot: bool,
        // db: Db
    }
}

impl Stream for GaggimateListener {
    type Item = Result<Uuid>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            let Some(item) = ready!(this.inner.as_mut().try_poll_next(cx)).transpose()? else {
                return Poll::Ready(None);
            };
            match item {
                GmMessage::Status(s) => this.current_state.merge(s),
                GmMessage::Unknown => eprintln!("encountered unknown message type, continuing..."),
            }
            // detect shot start + end
            // record in db
            // return ready with Uuid
        }
    }
}

pin_project! {
    pub struct GaggimateStream<S> {
        #[pin]
        inner: S
    }
}

impl<S> GaggimateStream<S> {
    pub fn new(inner: S) -> Self {
        GaggimateStream { inner }
    }
}

impl GaggimateStream<WsStream> {
    pub async fn connect(url: &Url) -> Result<Self> {
        let (stream, _response) = connect_async(url).await?;
        Ok(Self { inner: stream })
    }
}

impl<S> Stream for GaggimateStream<S>
where
    S: TryStream<Ok = WsMessage, Error = TungError>,
{
    type Item = Result<GmMessage>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use WsMessage::*;
        let mut this = self.project();
        loop {
            let Some(item) = ready!(this.inner.as_mut().try_poll_next(cx)).transpose()? else {
                return Poll::Ready(None);
            };

            match item {
                // let the underlying tokio-tungstenite stream handle these
                Ping(_) | Pong(_) | Close(_) | Frame(_) => continue,
                Text(m) => return Poll::Ready(Some(serde_json::from_str(&m).map_err(Into::into))),
                Binary(b) => {
                    return Poll::Ready(Some(serde_json::from_slice(&b).map_err(Into::into)));
                }
            }
        }
    }
}

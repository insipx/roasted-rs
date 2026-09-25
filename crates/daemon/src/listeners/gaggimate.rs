use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use color_eyre::eyre::{Report, Result};
use futures::{Stream, TryStream};
use pin_project_lite::pin_project;
use roasted_types::gaggimate::Message as GmMessage;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message as WsMessage, error::Error as TungError},
};
use url::Url;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Listener {
    url: Url,
    stream: GaggimateStream<WsStream>,
}

impl Listener {
    /// Connect to the Gaggimate Websockets Stream
    pub async fn connect(url: Url) -> Result<Self> {
        Ok(Self { stream: GaggimateStream::connect(&url).await?, url })
    }
}

// pub async fn listener() -> Result<()> {
//     while let Some(Ok(ev)) = read.next().await {
//         let msg = decode_message(ev)?;
//         println!("{:#?}", msg);
//     }
//     Ok(())
// }

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
            let item = ready!(this.inner.as_mut().try_poll_next(cx));
            let Some(item) = item else {
                return Poll::Ready(None);
            };
            let Ok(item) = item else { return Poll::Ready(Some(Err(item.unwrap_err().into()))) };

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

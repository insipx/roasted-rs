use std::{
    pin::Pin,
    task::{Context, Poll},
};

use color_eyre::Result;
use futures::Stream;
use pin_project_lite::pin_project;
use postcard::accumulator::{CobsAccumulator, FeedResult};
use roasted_types::zippy::ZippyResponse;
use tokio::io::{AsyncRead, ReadBuf};
use tokio_serial::SerialStream;

pin_project! {
    pub struct CobsStream {
        #[pin]
        inner: SerialStream,
        buffer: [u8; 1024],
        accumulator: CobsAccumulator<256>,
        pending_len: usize,
    }
}

impl CobsStream {
    pub fn new(inner: SerialStream) -> Self {
        Self { inner, buffer: [0u8; 1024], accumulator: CobsAccumulator::new(), pending_len: 0 }
    }
}

impl Stream for CobsStream {
    type Item = Result<ZippyResponse>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            let mut buffer = ReadBuf::new(this.buffer.as_mut());
            if *this.pending_len == 0 {
                let _ = futures::ready!(this.inner.as_mut().poll_read(cx, &mut buffer))?;
            } else {
                buffer.set_filled(std::mem::take(this.pending_len));
            }
            let ct = buffer.filled().len();
            // Finished reading input
            if ct == 0 {
                return Poll::Ready(None);
            }

            let buf = &this.buffer[..ct];
            let mut window = &buf[..];

            'cobs: while !window.is_empty() {
                window = match this.accumulator.feed::<ZippyResponse>(&window) {
                    FeedResult::Consumed => break 'cobs,
                    FeedResult::OverFull(new_wind) => new_wind,
                    FeedResult::DeserError(new_wind) => {
                        eprintln!("deserialization failed");
                        new_wind
                    },
                    FeedResult::Success { data, remaining } => {
                        let len = remaining.len();
                        let start = ct - len;
                        this.buffer.copy_within(start..ct, 0);
                        *this.pending_len = len;
                        return Poll::Ready(Some(Ok(data)));
                    }
                };
            }
        }
    }
}

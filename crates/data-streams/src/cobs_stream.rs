//! Cobs delineated streams for Zippy Response and Request data types
use alloc::boxed::Box;
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "std")]
use embedded_io_adapters::tokio_1::FromTokio;
use embedded_io_async::{ErrorType, Read};
use futures_core::Stream;
use futures_util::TryStream;
use postcard::accumulator::{CobsAccumulator, FeedResult};

use crate::error::{DataStreamError, Result};

// TODO: need a [`Sink`] implementation for writing
/// A Stream of some [`roasted-types`] `zippy` Request or Response type.
/// Compatible with embedded and std environments, to be used over UART, JTAG
/// or TCP/WebSockets
/// Internally uses [`futures_util::stream::unfold`] and is cancellation-safe.
/// if a `next` call is dropped, any bytes or in-progress deserialization progress
/// is still kept for the next call.
pub struct ZippyStream<T> {
    inner: Pin<Box<dyn Stream<Item = Result<T>>>>,
}

impl<T> Stream for ZippyStream<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().inner.as_mut().try_poll_next(cx)
    }
}

impl<T: 'static> ZippyStream<T>
where
    T: for<'de> serde::de::Deserialize<'de>,
{
    pub fn new_embedded<S>(s: S) -> Self
    where
        S: Read + 'static,
        <S as ErrorType>::Error: 'static,
    {
        let stream = InnerCobsStream::<S, T, 128>::new(s);
        Self { inner: Box::pin(stream.into_stream()) }
    }

    #[cfg(feature = "std")]
    pub fn new_tokio<Tk>(inner: Tk) -> Self
    where
        Tk: tokio::io::AsyncRead + Unpin + 'static,
    {
        let stream = embedded_io_adapters::tokio_1::FromTokio::new(inner);
        let stream = InnerCobsStream::<FromTokio<Tk>, T, 1024>::new(stream);
        Self { inner: Box::pin(stream.into_stream()) }
    }
}

struct InnerCobsStream<S, T, const N: usize> {
    inner: S,
    buffer: [u8; N],
    accumulator: CobsAccumulator<256>,
    pending_len: usize,
    filled: usize,
    _marker: PhantomData<T>,
}

impl<S, T, const N: usize> InnerCobsStream<S, T, N> {
    fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: [0u8; N],
            accumulator: CobsAccumulator::new(),
            pending_len: 0,
            filled: 0,
            _marker: PhantomData,
        }
    }
}

impl<S, T, const N: usize> InnerCobsStream<S, T, N>
where
    S: Read,
    <S as ErrorType>::Error: 'static,
    T: for<'de> serde::de::Deserialize<'de>,
{
    pub fn into_stream(self) -> impl Stream<Item = Result<T>> {
        futures_util::stream::try_unfold(self, |mut state| async move {
            let item = state.next_frame().await?;
            Ok(item.map(|i| (i, state)))
        })
    }

    async fn next_frame(&mut self) -> Result<Option<T>> {
        loop {
            let buffer = self.buffer.as_mut();
            if self.pending_len == 0 {
                self.filled = self
                    .inner
                    .read(buffer)
                    .await
                    .map_err(|e| DataStreamError::ReadFailed(format!("{:?}", e)))?;
            } else {
                self.filled = core::mem::take(&mut self.pending_len);
            }
            let ct = self.filled;
            // Finished reading input
            if ct == 0 {
                return Ok(None);
            }

            let buf = &self.buffer[..ct];
            let mut window = buf;

            'cobs: while !window.is_empty() {
                window = match self.accumulator.feed::<T>(window) {
                    FeedResult::Consumed => break 'cobs,
                    FeedResult::OverFull(new_wind) => new_wind,
                    FeedResult::DeserError(new_wind) => new_wind,
                    FeedResult::Success { data, remaining } => {
                        let len = remaining.len();
                        let start = ct - len;
                        self.buffer.copy_within(start..ct, 0);
                        self.pending_len = len;
                        return Ok(Some(data));
                    }
                };
            }
        }
    }
}

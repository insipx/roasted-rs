use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use roasted_types::zippy::{CliAction, ZippyResponse};

use crate::{cobs_stream::ZippyStream, error::Result};

macro_rules! zippy_stream {
    ($(#[$meta:meta])* $vis:vis struct $name:ident($item:ty);) => {
        $(#[$meta])*
        $vis struct $name {
            inner: ZippyStream<$item>,
        }

        impl $name {
            pub fn new_embedded<S>(inner: S) -> Self
            where
                S: embedded_io_async::Read + 'static,
                <S as embedded_io_async::ErrorType>::Error: 'static,
            {
                Self { inner: ZippyStream::new_embedded(inner) }
            }

            #[cfg(feature = "std")]
            pub fn new_tokio<S>(inner: S) -> Self
            where
                S: tokio::io::AsyncRead + Unpin + 'static,
            {
                Self { inner: ZippyStream::new_tokio(inner) }
            }
        }

        impl Stream for $name {
            type Item = Result<$item>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                Pin::new(&mut self.get_mut().inner).poll_next(cx)
            }
        }
    };
}

zippy_stream! {
    /// COBS-framed actions sent to Zippy.
    /// Internally uses [`futures_util::stream::unfold`] and is cancellation-safe.
    /// if a `next` call is dropped, any bytes or in-progress deserialization progress
    /// is still kept for the next call.
    pub struct ActionStream(CliAction);
}

zippy_stream! {
    /// COBS-framed responses received from Zippy.
   /// Internally uses [`futures_util::stream::unfold`] and is cancellation-safe.
    /// if a `next` call is dropped, any bytes or in-progress deserialization progress
    /// is still kept for the next call.
    pub struct ResponseStream(ZippyResponse);
}

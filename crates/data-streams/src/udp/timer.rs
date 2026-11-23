#[cfg(not(feature = "std"))]
use embassy_time::{Duration, Instant};
#[cfg(feature = "std")]
use {
    ::tokio::time::{Instant, Sleep, sleep_until},
    alloc::boxed::Box,
    core::{future::Future, pin::Pin, time::Duration},
};

use super::{Context, DtlsWakerContext, RefCell, c_int, c_void};

pub(super) struct Delay {
    intermediate: Instant,
    final_deadline: Instant,
    notified: bool,
    #[cfg(feature = "std")]
    sleep: Option<Pin<Box<Sleep>>>,
}

// in case we wake up before the deadline (b/c this is called from inside stream/sink)
// we may need to update the saved/scheduled wake.
// or if the deadline expired and we need to set a new deadline from the timer
// embassy only keeps the earliest alarm
impl Delay {
    pub(super) fn poll_wake(&mut self, cx: &mut Context<'_>) {
        if self.notified {
            return;
        }
        if Instant::now() >= self.final_deadline {
            // The deadline may have passed just after Mbed TLS checked it.
            self.notified = true;
            cx.waker().wake_by_ref();
            return;
        }

        #[cfg(not(feature = "std"))]
        embassy_time_driver::schedule_wake(self.final_deadline.as_ticks(), cx.waker());

        #[cfg(feature = "std")]
        {
            let sleep =
                self.sleep.get_or_insert_with(|| Box::pin(sleep_until(self.final_deadline)));
            if sleep.as_mut().poll(cx).is_ready() {
                self.notified = true;
                cx.waker().wake_by_ref();
            }
        }
    }
}

// https://docs.rs/mbedtls-rs-sys/latest/mbedtls_rs_sys/type.mbedtls_ssl_set_timer_t.html
pub(super) unsafe extern "C" fn set_delay(
    context: *mut c_void,
    intermediate_ms: u32,
    final_ms: u32,
) {
    let context = unsafe { &*context.cast::<RefCell<DtlsWakerContext<'_>>>() };
    let mut context = context.borrow_mut();
    if final_ms == 0 {
        context.delay = None;
        return;
    }

    let now = Instant::now();
    let final_deadline = now + Duration::from_millis(u64::from(final_ms));
    context.delay = Some(Delay {
        intermediate: now + Duration::from_millis(u64::from(intermediate_ms)),
        final_deadline,
        notified: false,
        #[cfg(feature = "std")]
        sleep: None,
    });

    #[cfg(not(feature = "std"))]
    if let Some(waker) = &context.waker {
        embassy_time_driver::schedule_wake(final_deadline.as_ticks(), waker);
    }
}

// docs.rs/mbedtls-rs-sys/latest/mbedtls_rs_sys/type.mbedtls_ssl_get_timer_t.html
pub(super) unsafe extern "C" fn get_delay(context: *mut c_void) -> c_int {
    let context = unsafe { &*context.cast::<RefCell<DtlsWakerContext<'_>>>() };
    let context = context.borrow();
    let Some(delay) = &context.delay else {
        return -1;
    };

    let now = Instant::now();
    if now >= delay.final_deadline {
        2
    } else if now >= delay.intermediate {
        1
    } else {
        0
    }
}

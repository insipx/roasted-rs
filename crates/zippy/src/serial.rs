use embedded_io_async::Write;
use esp_backtrace as _;
use esp_hal::{
    Async,
    usb::usb_serial_jtag::{UsbSerialJtagRx, UsbSerialJtagTx},
};
use postcard::{
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use roasted_types::zippy::{CliAction, ZippyResponse};

use crate::{configuration::MAX_BUFFER_SIZE, error::Result};

#[embassy_executor::task]
pub async fn cli_driver(rx: UsbSerialJtagRx<'static, Async>, tx: UsbSerialJtagTx<'static, Async>) {
    if let Err(e) = run(rx, tx).await {
        panic!("CLI Command Task Failed {e:?}");
    }
}

async fn run(
    mut rx: UsbSerialJtagRx<'static, Async>,
    mut tx: UsbSerialJtagTx<'static, Async>,
) -> Result<()> {
    let mut rbuf = [0u8; MAX_BUFFER_SIZE];
    loop {
        let r = embedded_io_async::Read::read(&mut rx, &mut rbuf).await;
        match r {
            Ok(len) => {
                if let Some(response) = process_action(&rbuf[..len])? {
                    send(response, &mut tx).await?;
                }
                send(ZippyResponse::End, &mut tx).await?;
            }
            #[allow(unreachable_patterns)]
            Err(e) => esp_println::println!("RX Error: {:?}", e),
        }
    }
}

async fn send(response: ZippyResponse, tx: &mut UsbSerialJtagTx<'static, Async>) -> Result<()> {
    let mut reply_buffer = [0u8; core::mem::size_of::<ZippyResponse>()];
    let res = serialize_with_flavor::<_, Cobs<Slice>, &mut [u8]>(
        &response,
        Cobs::try_new(Slice::new(&mut reply_buffer[6..]))?,
    )?;
    tx.write_all(res).await.expect("infallible");
    Ok(())
}

fn process_action(rbuf: &[u8]) -> Result<Option<ZippyResponse>> {
    use CliAction::*;
    let mut action_buffer: heapless::Vec<_, MAX_BUFFER_SIZE> = heapless::Vec::new();
    action_buffer.extend_from_slice(rbuf)?;
    let action: CliAction = postcard::from_bytes(&action_buffer)?;

    Ok(match action {
        SayHello => Some(new_message("Hello, this is Zippy!")?),
        InitWifi(_) => todo!(),
        _ => None,
    })
}

fn new_message(msg: &str) -> Result<ZippyResponse> {
    let mut buf = [0u8; 32];
    let len = msg.len();
    if len >= 32 {
        buf.copy_from_slice(&msg.as_bytes()[..32]);
    } else {
        buf[0..len].copy_from_slice(&msg.as_bytes()[..len]);
    }
    Ok(ZippyResponse::Message(buf))
}

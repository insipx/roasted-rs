use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::Async;
use postcard::{
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use roasted_types::zippy::{CliAction, ZippyResponse};

use crate::{configuration::MAX_BUFFER_SIZE, error::Result};

#[cfg(bare_metal)]
pub type CommsPeripheral = esp_hal::usb::usb_serial_jtag::UsbSerialJtag<'static, Async>;

#[cfg(emulated)]
pub type CommsPeripheral = esp_hal::uart::Uart<'static, Async>;

#[embassy_executor::task]
pub async fn cli_driver(rw: CommsPeripheral) {
    esp_println::println!("running CLI Driver\r");
    if let Err(e) = run(rw).await {
        panic!("CLI Command Task Failed {e:?}");
    }
}

async fn run<D>(mut rw: D) -> Result<()>
where
    D: Read + Write,
{
    let mut rbuf = [0u8; MAX_BUFFER_SIZE];
    loop {
        let r = embedded_io_async::Read::read(&mut rw, &mut rbuf).await;
        match r {
            Ok(len) => {
                esp_println::println!("got an action\r");
                if let Some(response) = process_action(&rbuf[..len])? {
                    send(response, &mut rw).await?;
                }
                send(ZippyResponse::End, &mut rw).await?;
            }
            #[allow(unreachable_patterns)]
            Err(e) => esp_println::println!("RX Error: {:?}", e),
        }
    }
}

async fn send<T>(response: ZippyResponse, tx: &mut T) -> Result<()>
where
    T: Write,
{
    let mut reply_buffer = [0u8; core::mem::size_of::<ZippyResponse>()];
    let res = serialize_with_flavor::<_, Cobs<Slice>, &mut [u8]>(
        &response,
        Cobs::try_new(Slice::new(&mut reply_buffer))?,
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

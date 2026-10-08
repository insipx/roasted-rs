use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::Async;
use postcard::{
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use roasted_types::zippy::{CliAction, ZippyResponse};
use crate::alloc::string::ToString;
use crate::{configuration::MAX_BUFFER_SIZE, error::Result};

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
                if let Some(response) = process_action(&rbuf[..len])? {
                    esp_println::println!("sending {:?}", response);
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
    let mut reply_buffer = [0u8; 64];
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
    let action: CliAction = postcard::from_bytes_cobs(action_buffer.as_mut_slice())?;
    esp_println::println!("action: {action:?}\r");


    Ok(match action {
        SayHello => Some(new_message("Hello, this is Zippy!")?),
        InitWifi(_) => todo!(),
        _ => None,
    })
}

fn new_message(msg: &str) -> Result<ZippyResponse> {
    Ok(ZippyResponse::Message(msg.to_string()))
}

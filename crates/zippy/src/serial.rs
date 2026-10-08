use bon::Builder;
use embassy_sync::channel::DynamicSender;
use esp_backtrace as _;
use esp_hal::Async;
use roasted_types::zippy::CliAction;

use crate::{configuration::MAX_BUFFER_SIZE, error::Result};

#[derive(Builder)]
pub struct UartCli {
    uart_rx: esp_hal::uart::UartRx<'static, Async>,
    // internal message sender
    tx: DynamicSender<'static, CliAction>,
}

#[embassy_executor::task]
pub async fn cli_driver(uart: UartCli) {
    esp_println::println!("running CLI Driver\r");
    if let Err(e) = run(uart).await {
        panic!("CLI Command Task Failed {e:?}");
    }
}

async fn run(mut rw: UartCli) -> Result<()> {
    let mut rbuf = [0u8; MAX_BUFFER_SIZE];
    loop {
        let r = embedded_io_async::Read::read(&mut rw.uart_rx, &mut rbuf).await;
        match r {
            Ok(len) => {
                let action = process_action(&rbuf[..len])?;
                rw.tx.send(action).await;
            }
            #[allow(unreachable_patterns)]
            Err(e) => esp_println::println!("RX Error: {:?}", e),
        }
    }
}

fn process_action(rbuf: &[u8]) -> Result<CliAction> {
    use CliAction::*;
    let mut action_buffer: heapless::Vec<_, MAX_BUFFER_SIZE> = heapless::Vec::new();
    action_buffer.extend_from_slice(rbuf)?;
    let action: CliAction = postcard::from_bytes_cobs(action_buffer.as_mut_slice())?;
    Ok(action)
    // esp_println::println!("action: {action:?}\r");
    //
    // Ok(match action {
    //     SayHello => Some(new_message("Hello, this is Zippy!")?),
    //     Write(s) => todo!(),
    //     InitWifi(_) => todo!(),
    //     _ => None,
    // })
}

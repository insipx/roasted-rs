//! Processes actions
use alloc::string::ToString;

use bon::Builder;
use embassy_sync::channel::DynamicReceiver;
use embedded_io_async::Write;
use esp_bootloader_esp_idf::partitions::FlashStorage;
use esp_hal::{Async, rng::TrngSource, uart::UartTx};
use esp_println::println;
use futures::TryStreamExt;
use postcard::{ser_flavors::{Cobs, Slice}, serialize_with_flavor};
use roasted_types::zippy::{CliAction, ZippyResponse};

use crate::error::Result;

#[derive(Builder)]
pub struct CommandExecutor {
    db: FlashStorage<'static>,
    entropy: TrngSource<'static>,
    uart_tx: UartTx<'static, Async>,
    // internal msg channel
    rx: DynamicReceiver<'static, CliAction>,
}

#[embassy_executor::task]
pub async fn run(mut commands: CommandExecutor) {
    loop {
        if let Err(e) = commands.next().await {
            println!("{e}");
        }
    }
}

impl CommandExecutor {
    pub async fn next(&mut self) -> Result<()> {
        let action = self.rx.receive().await;
        // match action {
        //     SayHello => Some()
        //
        // }
        Ok(())
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

fn new_message(msg: &str) -> Result<ZippyResponse> {
    Ok(ZippyResponse::Message(msg.to_string()))
}
// send(ZippyResponse::End, &mut rw.uart_rx).await?;

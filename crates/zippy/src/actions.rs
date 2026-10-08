//! Processes actions
use alloc::string::{String, ToString};

use bon::bon;
use embassy_sync::channel::DynamicReceiver;
use embedded_io_async::Write;
use esp_bootloader_esp_idf::partitions::FlashStorage;
use esp_hal::{Async, rng::TrngSource, uart::UartTx};
use esp_println::println;
use postcard::{
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use roasted_types::zippy::{CliAction, ZippyResponse};

use crate::{
    db::{self, Database},
    error::Result,
};

pub struct CommandExecutor {
    db: Database<'static, String>,
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

#[bon]
impl CommandExecutor {
    #[builder]
    pub fn new(
        db: FlashStorage<'static>,
        entropy: TrngSource<'static>,
        uart_tx: UartTx<'static, Async>,
        rx: DynamicReceiver<'static, CliAction>,
    ) -> Result<Self> {
        Ok(Self { db: Database::new(db, entropy)?, uart_tx, rx })
    }

    pub async fn next(&mut self) -> Result<()> {
        let action = self.rx.receive().await;
        println!("Processing action {action:?}");
        match action {
            CliAction::SayHello => {
                send(new_message("Hello, this is Zippy!")?, &mut self.uart_tx).await?;
                send(ZippyResponse::End, &mut self.uart_tx).await?;
            }
            CliAction::Write(s) => {
                self.db.put(db::Keys::MISC, s);
                self.db.flush()?;
            }
            CliAction::InitWifi(_) => todo!(),
            CliAction::ListPorts => todo!(),
            CliAction::SetDaemonUrl(_) => todo!(),
        }
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

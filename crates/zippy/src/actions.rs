//! Processes actions received over internal message channel
use alloc::string::{String, ToString};

use anyhow::Result;
use bon::bon;
use embassy_sync::channel::DynamicReceiver;
use embedded_io_async::Write;
use esp_hal::{Async, uart::UartTx};
use esp_println::println;
use postcard::{
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};
use roasted_types::zippy::{CliAction, InitWifi, ZippyResponse};

use crate::db::{self, Database};

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
        db: Database<'static, String>,
        uart_tx: UartTx<'static, Async>,
        rx: DynamicReceiver<'static, CliAction>,
    ) -> Result<Self> {
        Ok(Self { db, uart_tx, rx })
    }

    pub async fn next(&mut self) -> Result<()> {
        let action = self.rx.receive().await;
        println!("Processing action");
        match action {
            CliAction::SayHello => {
                self.send(new_message("Hello, this is Zippy!")?).await?;
            }
            CliAction::InitWifi(w) => {
                let InitWifi { ssid, password } = w;
                self.db.put(db::Keys::SSID, ssid).await;
                self.db.put(db::Keys::WIPW, password).await;
                if let Err(e) = self.db.flush().await {
                    self.send(new_emessage(alloc::format!("Error while saving wifi to db {e}"))?)
                        .await?;
                }
                self.send(new_message("written")?).await?;
            }
            CliAction::ListPorts => todo!(),
            CliAction::SetDaemonUrl(u) => {
                self.db.put(db::Keys::DURL, u.to_string()).await;
                if let Err(e) = self.db.flush().await {
                    self.send(new_emessage(alloc::format!("Error saving daemon url {e}"))?).await?;
                }
                self.send(new_message(alloc::format!("wrote {u}"))?).await?;
            }
        }
        self.end().await?;
        Ok(())
    }

    async fn end(&mut self) -> Result<()> {
        self.send(ZippyResponse::End).await?;
        Ok(())
    }

    async fn send(&mut self, response: ZippyResponse) -> Result<()> {
        let mut reply_buffer = [0u8; 1024];
        let res = serialize_with_flavor::<_, Cobs<Slice>, &mut [u8]>(
            &response,
            Cobs::try_new(Slice::new(&mut reply_buffer))?,
        )?;
        self.uart_tx.write_all(res).await.expect("infallible");
        Ok(())
    }
}

fn new_message<S: AsRef<str>>(msg: S) -> Result<ZippyResponse> {
    let s: &str = msg.as_ref();
    Ok(ZippyResponse::Message(s.to_string()))
}

/// Create an error message
fn new_emessage<S: AsRef<str>>(msg: S) -> Result<ZippyResponse> {
    let s: &str = msg.as_ref();
    Ok(ZippyResponse::ErrorMessage(s.to_string()))
}

// send(ZippyResponse::End, &mut rw.uart_rx).await?;

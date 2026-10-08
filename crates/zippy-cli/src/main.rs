use std::time::Duration;

use cobs_stream::CobsStream;
use color_eyre::{Result, eyre::bail};
use futures::TryStreamExt;
use roasted_types::zippy::{CliAction, ZippyResponse};
use tokio::io::AsyncWriteExt;
use tokio_serial::SerialPortBuilderExt;

mod actions;
mod args;
mod cobs_stream;

#[tokio::main(flavor = "local")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = args::parse_args()?;
    let args::Args { ref action, .. } = args;

    match action {
        CliAction::ListPorts => {
            actions::list_ports::run()?;
            std::process::exit(0);
        }
        _ => (),
    }
    drive(args).await?;

    Ok(())
}

async fn drive(args: args::Args) -> Result<()> {
    let args::Args { device, action } = args;
    let Some(device) = device else { bail!("`device` must be specified with `-d` or `--device`") };

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);
    let mut port = tokio_serial::new(device.to_string_lossy(), 115200)
        .timeout(Duration::from_secs(5))
        .open_native_async()?;

    let mut buffer = [0u8; std::mem::size_of::<CliAction>()];
    let frame = postcard::to_slice_cobs(&action, &mut buffer)?;
    port.write_all(frame).await?;
    port.flush().await?;

    let mut s = CobsStream::new(port);
    loop {
        tokio::select! {
            response = s.try_next() => {
                if let Some(response) = response? {
                    match response {
                        ZippyResponse::End => break println!("end"),
                        ZippyResponse::Message(message) => println!("{}", &message)
                    }
                }
            }
            result = &mut shutdown => {
                result?;
                break;
            }
        }
    }

    Ok(())
}

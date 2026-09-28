use color_eyre::{Result, eyre::bail};
use roasted_types::zippy::CliAction;
use serialport::SerialPortType;

use crate::args::Args;

mod actions;
mod args;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = args::parse_args()?;
    let args::Args { ref device, ref action } = args;

    match action {
        CliAction::SayHello => {
            ensure_args(&args);
            todo!()
        }
        CliAction::InitWifi(_) => {
            todo!()
        }
        CliAction::ListPorts => {
            actions::list_ports::run()?;
        }
    }
    Ok(())
}

fn ensure_args(args: &Args) -> Result<()> {
    if args.device.is_none() {
        bail!("`device` must be specified with `-d` or `--device`")
    }
    Ok(())
}

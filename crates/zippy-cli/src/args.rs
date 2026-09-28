use serde::{Serialize, Deserialize};
use roasted_types::zippy::{InitWifi, CliAction};
use color_eyre::{Result, Report};
use color_eyre::eyre::{WrapErr, bail};
use std::path::PathBuf;
use lexopt::{Parser, prelude::*};
use dialoguer::Password;

use std::str::FromStr;

fn err(name: &str) -> String {
    format!("failed to parse argument `{name}`")
}

#[derive(Clone, Debug, Default)]
pub struct Args {
    pub device: PathBuf,
    pub action: CliAction
}

/// The CLI Action for lexopt. Must reflect [`CliAction`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
enum Command {
    #[default]
    SayHello,
    InitWifi
}

impl FromStr for Command {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SayHello" | "sayhello" | "hello" | "say_hello" => {
                Ok(Self::SayHello)
            },
            "InitWifi" | "init_wifi" | "initwifi" | "wifi" => {
                Ok(Self::InitWifi)
            },
            _ => bail!("unknown command: `{}` for `CliAction`", s)
        }
    }
}

fn parse_hello(parser: &mut Parser) -> CliAction {
    CliAction::SayHello
}

fn parse_wifi(parser: &mut Parser) -> Result<CliAction> {
    let mut ssid = None;

    while let Some(args) = parser.next()? {
        match args {
            Short('s') | Long("ssid") => {
                ssid = Some(parser.value()?.parse().wrap_err(err("ssid"))?);
            },
            _ => bail!(args.unexpected())
        }
    }

    let Some(ssid) = ssid else {
        bail!("`ssid` must be specified with `-s` or `--ssid`");
    };

    let password = Password::new()
      .with_prompt("Wi-Fi password")
      .allow_empty_password(true)
      .interact()?;

    Ok(CliAction::InitWifi(InitWifi {
        ssid, password
    }))
}

pub fn parse_args() -> Result<Args> {
    let mut parser = lexopt::Parser::from_env();

    let mut device: Option<PathBuf> = None;
    let mut action = None;
    while let Some(arg) = parser.next()? {
        match arg {
            Short('d') | Long("device") => {
                device = Some(parser.value()?.parse().wrap_err(err("device"))?);
            },
            Value(val) => {
                if device.is_none() {
                    break;
                }
                let cmd = Command::from_str(&val.string()?)?;
                action = match cmd {
                    Command::SayHello => Some(parse_hello(&mut parser)),
                    Command::InitWifi => Some(parse_wifi(&mut parser)?),
                };
                break;
            },
            Short('h') | Long("help") => {
                println!(
                    "Usage: zippy-cli [-d|--device=STRING -a|--action `say_hello|init_wifi`]"
                );
                std::process::exit(0);
            }
            _ => bail!(arg.unexpected()),
        }
    }

    let action = action.unwrap_or(CliAction::SayHello);
    let Some(device) = device else {
        bail!("`device` must be specified with `-d` or `--device`")
    };

    Ok(Args {
        device,
        action
    })
}


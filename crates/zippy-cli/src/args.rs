use std::{path::PathBuf, str::FromStr};

use color_eyre::{
    Report, Result,
    eyre::{WrapErr, bail},
};
use dialoguer::Password;
use lexopt::{Parser, prelude::*};
use roasted_types::zippy::{CliAction, InitWifi};

fn err(name: &str) -> String {
    format!("failed to parse argument `{name}`")
}

#[derive(Clone, Debug, Default)]
pub struct Args {
    pub device: Option<PathBuf>,
    pub action: CliAction,
}

/// The CLI Action for lexopt. Must reflect [`CliAction`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
enum Command {
    #[default]
    SayHello,
    InitWifi,
    /// List available ports
    ListPorts,
}

impl FromStr for Command {
    type Err = Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SayHello" | "sayhello" | "hello" | "say-hello" | "say_hello" => Ok(Self::SayHello),
            "InitWifi" | "init_wifi" | "init-wifi" | "initwifi" | "wifi" => Ok(Self::InitWifi),
            "ListPorts" | "list_ports" | "list-ports" | "listports" | "ports" => {
                Ok(Self::ListPorts)
            }
            _ => bail!("unknown command: `{}` for `CliAction`", s),
        }
    }
}

fn parse_wifi(parser: &mut Parser) -> Result<CliAction> {
    let mut ssid = None;

    while let Some(args) = parser.next()? {
        match args {
            Short('s') | Long("ssid") => {
                ssid = Some(parser.value()?.parse().wrap_err(err("ssid"))?);
            }
            Short('h') | Long("help") => {
                println!("Usage: zippy-cli [-d|--device STRING] init-wifi [-s|--ssid=STRING]");
                std::process::exit(0);
            }
            _ => bail!(args.unexpected()),
        }
    }

    let Some(ssid) = ssid else {
        bail!("`ssid` must be specified with `-s` or `--ssid`");
    };

    let password =
        Password::new().with_prompt("Wi-Fi password").allow_empty_password(true).interact()?;

    Ok(CliAction::InitWifi(InitWifi::builder().ssid(ssid).password(password).build()))
}

pub fn parse_args() -> Result<Args> {
    let mut parser = lexopt::Parser::from_env();

    let mut device: Option<PathBuf> = None;
    let mut action = None;
    while let Some(arg) = parser.next()? {
        match arg {
            Short('d') | Long("device") => {
                device = Some(parser.value()?.parse().wrap_err(err("device"))?);
            }
            Value(val) => {
                let cmd = Command::from_str(&val.string()?)?;
                action = match cmd {
                    Command::SayHello => Some(CliAction::SayHello),
                    Command::InitWifi => Some(parse_wifi(&mut parser)?),
                    Command::ListPorts => Some(CliAction::ListPorts),
                };
                break;
            }
            Short('h') | Long("help") => {
                println!("Usage: zippy-cli [-d|--device=FILE [ACTION]");
                std::process::exit(0);
            }
            _ => bail!(arg.unexpected()),
        }
    }

    let action = action.unwrap_or(CliAction::SayHello);

    Ok(Args { device, action })
}

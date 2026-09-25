//! CLI Args for the daemon

use color_eyre::eyre::{Result, WrapErr, bail};
use url::Url;

#[derive(Clone)]
pub struct Args {
    /// Url for Gaggimate
    pub gaggimate: Url,
    /// port to bind webserver on, default 443
    pub ws_port: u16,
    /// port to bind webserver on to accept audio from esp32.
    /// Default 5005
    pub udp_port: u16,
    /// Url. when given will send application logs to Loki.
    pub loki: Option<Url>,
}

/// Parse the static website directory from the command line
pub fn parse_args() -> Result<Args> {
    use lexopt::prelude::*;

    let mut gaggimate = None;
    let mut ws_port = 80;
    let mut udp_port = 5005;
    let mut loki = None;
    let mut parser = lexopt::Parser::from_env();
    let err = |name| format!("failed to parse argument `{name}`");
    while let Some(arg) = parser.next()? {
        match arg {
            Short('g') | Long("gaggimate") => {
                gaggimate = Some(parser.value()?.parse().wrap_err(err("gaggimate"))?);
            }
            Long("loki") => {
                loki = Some(parser.value()?.parse().wrap_err(err("loki"))?);
            }
            Long("ws_port") => {
                ws_port = parser.value()?.parse().wrap_err(err("ws_port"))?;
            }
            Long("udp_port") => {
                udp_port = parser.value()?.parse().wrap_err(err("udp_port"))?;
            }
            Short('h') | Long("help") => {
                println!(
                    "Usage: roasted-daemon [-g|--gaggimate=STRING --ws-port=PORT --udp_port=PORT --loki=URL]"
                );
                std::process::exit(0);
            }
            _ => bail!(arg.unexpected()),
        }
    }

    if gaggimate.is_none() {
        panic!("no gaggimate URL provided")
    }

    Ok(Args { gaggimate: gaggimate.unwrap(), ws_port, udp_port, loki })
}

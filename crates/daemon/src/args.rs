//! CLI Args for the daemon

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
pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut gaggimate = None;
    let mut ws_port = 80;
    let mut udp_port = 5005;
    let mut loki = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('g') | Long("gaggimate") => {
                gaggimate = Some(parser.value()?.parse()?);
            }
            Long("loki") => {
                loki = Some(parser.value()?.parse()?);
            }
            Long("ws_port") => {
                ws_port = parser.value()?.parse()?;
            }
            Long("udp_port") => {
                udp_port = parser.value()?.parse()?;
            }
            Long("help") => {
                println!("Usage: srv [-d|--directory=STRING --loki=URL --port=NUM]");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    if gaggimate.is_none() {
        panic!("no gaggimate URL provided")
    }

    Ok(Args { gaggimate: gaggimate.unwrap(), ws_port, udp_port, loki })
}

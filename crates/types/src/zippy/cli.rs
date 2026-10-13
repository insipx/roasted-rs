use alloc::string::String;

use bon::Builder;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Possible actions to configure/interact with Zippy over a USB Serial interface
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CliAction {
    /// Just say hello
    #[default]
    SayHello,
    /// Initiate wifi sequence
    InitWifi(InitWifi),
    /// List available ports
    ListPorts,
    /// set the url of the roasted-rs service
    SetDaemonUrl(url::Url),
}

impl core::fmt::Display for CliAction {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CliAction::SayHello => write!(f, "SayHello"),
            CliAction::InitWifi(_) => write!(f, "InitWifi"),
            CliAction::ListPorts => write!(f, "ListPorts"),
            CliAction::SetDaemonUrl(_) => write!(f, "SetDaemonUrl")
        }
    }
}

/// Initializing Wifi
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Zeroize, Clone, Builder)]
pub struct InitWifi {
    /// Wifi Name
    pub ssid: String,
    /// Wifi password
    pub password: String,
}

/// A response from Zippy
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ZippyResponse {
    /// Zippy has nothing else to say
    #[default]
    End,
    /// Zippy encountered an error
    ErrorMessage(String),
    /// Bounded UTF-8 Message
    Message(String),
}

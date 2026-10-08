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
    /// Write a string to encrypted storage (for testing)
    Write(String),
    /// Initiate wifi sequence
    InitWifi(InitWifi),
    /// List available ports
    ListPorts,
    /// set the url of the roasted-rs service
    SetDaemonUrl(url::Url),
}

/// Initializing Wifi
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Zeroize, Clone, Builder)]
pub struct InitWifi {
    /// Wifi Name
    ssid: String,
    /// Wifi password
    password: String,
}

/// A response from Zippy
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ZippyResponse {
    /// Zippy has nothing else to say
    #[default]
    End,
    /// Bounded UTF-8 Message
    Message(String),
}

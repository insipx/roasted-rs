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
}

/// Initializing Wifi
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Zeroize, Clone, Builder)]
pub struct InitWifi {
    /// Wifi Name
    ssid: String,
    /// Wifi password
    password: String,
}

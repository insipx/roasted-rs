use zeroize::Zeroize;
use serde::{Serialize, Deserialize};
use bon::Builder;

/// Possible actions to configure/interact with Zippy over a USB Serial interface
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CliAction {
    /// Just say hello
    #[default]
    SayHello,
    /// Initiate wifi sequence
    InitWifi(InitWifi)
}

/// Initializing Wifi
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Zeroize, Clone, Builder)]
pub struct InitWifi {
    /// Wifi Name
    pub ssid: String,
    /// Wifi password
    pub password: String
}


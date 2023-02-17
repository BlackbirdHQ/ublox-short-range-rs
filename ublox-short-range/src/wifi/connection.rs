use crate::wifi::network::{WifiMode, WifiNetwork};

#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum WiFiState {
    Inactive,
    /// Searching for Wifi
    NotConnected,
    Connected,
}

/// Describes whether device is connected to a network and has an IP or not.
/// It is possible to be attached to a network but have no Wifi connection.
#[derive(Debug, Clone, Copy, PartialEq, defmt::Format)]
pub enum NetworkState {
    Attached,
    AlmostAttached,
    Unattached,
}

// Fold into wifi connectivity
pub struct WifiConnection {
    /// Keeps track of connection state on module
    pub wifi_state: WiFiState,
    pub network_state: NetworkState,
    pub network: WifiNetwork,
    /// Numbre from 0-9. 255 used for unknown
    pub config_id: u8,
    /// Keeps track of activation of the config by driver
    pub activated: bool,
}

impl WifiConnection {
    pub(crate) fn new(network: WifiNetwork, wifi_state: WiFiState, config_id: u8) -> Self {
        WifiConnection {
            wifi_state,
            network_state: NetworkState::Unattached,
            network,
            config_id,
            activated: false,
        }
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.network_state == NetworkState::Attached && self.wifi_state == WiFiState::Connected
    }

    pub fn is_station(&self) -> bool {
        self.network.mode == WifiMode::Station
    }

    pub fn is_access_point(&self) -> bool {
        !self.is_station()
    }

    pub(crate) fn activate(mut self) -> Self {
        self.activated = true;
        self
    }

    pub(crate) fn deactivate(&mut self) {
        self.activated = false;
    }
}

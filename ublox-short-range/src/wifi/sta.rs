use crate::{
    client::UbloxClient,
    command::{
        edm::EdmAtCmdWrapper,
        wifi::{types::*, *},
        *,
    },
    error::{WifiConnectionError, WifiError},
    wifi::{
        connection::{WiFiState, WifiConnection},
        network::{WifiMode, WifiNetwork},
        options::ConnectionOptions,
    },
};
use atat::heapless_bytes::Bytes;
use atat::AtatClient;

use core::convert::{TryFrom, TryInto};
use embedded_hal::digital::OutputPin;
use embedded_time::duration::{Generic, Milliseconds};
use embedded_time::Clock;
use heapless::Vec;

/// Wireless network connectivity functionality.
pub trait WifiConnectivity {
    /// Makes an attempt to connect to a selected wireless network with password specified.
    fn connect(&mut self, options: ConnectionOptions) -> Result<(), WifiConnectionError>;

    fn scan(&mut self) -> Result<Vec<WifiNetwork, 32>, WifiError>;

    fn is_connected(&self) -> bool;

    fn disconnect(&mut self) -> Result<(), WifiConnectionError>;
}

impl<C, CLK, RST, const N: usize, const L: usize> WifiConnectivity
    for UbloxClient<C, CLK, RST, N, L>
where
    C: AtatClient,
    CLK: Clock,
    RST: OutputPin,
    Generic<CLK::T>: TryInto<Milliseconds>,
{
    /// Attempts to connect to a wireless network with the given connection options.
    fn connect(&mut self, options: ConnectionOptions) -> Result<(), WifiConnectionError> {
        let config_id = options.config_id.unwrap_or(0);

        // Network part
        // Deactivate network id 0
        self.send_internal(
            &EdmAtCmdWrapper(ExecWifiStationAction {
                config_id,
                action: WifiStationAction::Deactivate,
            }),
            true,
        )?;

        if let Some(ref con) = self.wifi_connection {
            if con.wifi_state != WiFiState::Inactive {
                return Err(WifiConnectionError::WaitingForWifiDeactivation);
            }
        }

        // Disable DHCP Client (static IP address will be used)
        if options.ip.is_some() || options.subnet.is_some() || options.gateway.is_some() {
            self.send_internal(
                &EdmAtCmdWrapper(SetWifiStationConfig {
                    config_id,
                    config_param: WifiStationConfig::IPv4Mode(IPv4Mode::Static),
                }),
                true,
            )?;
        }

        // Network IP address
        if let Some(ip) = options.ip {
            self.send_internal(
                &EdmAtCmdWrapper(SetWifiStationConfig {
                    config_id,
                    config_param: WifiStationConfig::IPv4Address(ip),
                }),
                true,
            )?;
        }
        // Network Subnet mask
        if let Some(subnet) = options.subnet {
            self.send_internal(
                &EdmAtCmdWrapper(SetWifiStationConfig {
                    config_id,
                    config_param: WifiStationConfig::SubnetMask(subnet),
                }),
                true,
            )?;
        }
        // Network Default gateway
        if let Some(gateway) = options.gateway {
            self.send_internal(
                &EdmAtCmdWrapper(SetWifiStationConfig {
                    config_id,
                    config_param: WifiStationConfig::DefaultGateway(gateway),
                }),
                true,
            )?;
        }

        // Active on startup
        // self.send_internal(&SetWifiStationConfig{
        //     config_id,
        //     config_param: WifiStationConfig::ActiveOnStartup(OnOff::On),
        // }, true)?;

        // Wifi part
        // Set the Network SSID to connect to
        self.send_internal(
            &EdmAtCmdWrapper(SetWifiStationConfig {
                config_id,
                config_param: WifiStationConfig::SSID(options.ssid.clone()),
            }),
            true,
        )?;

        if let Some(pass) = options.password.clone() {
            // Use WPA2 as authentication type
            self.send_internal(
                &EdmAtCmdWrapper(SetWifiStationConfig {
                    config_id,
                    config_param: WifiStationConfig::Authentication(Authentication::WpaWpa2Psk),
                }),
                true,
            )?;

            // Input passphrase
            self.send_internal(
                &EdmAtCmdWrapper(SetWifiStationConfig {
                    config_id,
                    config_param: WifiStationConfig::WpaPskOrPassphrase(pass),
                }),
                true,
            )?;
        }

        self.wifi_connection.replace(WifiConnection::new(
            WifiNetwork {
                bssid: Bytes::new(),
                op_mode: wifi::types::OperationMode::AdHoc,
                ssid: options.ssid,
                channel: 0,
                rssi: 1,
                authentication_suites: 0,
                unicast_ciphers: 0,
                group_ciphers: 0,
                mode: WifiMode::AccessPoint,
            },
            WiFiState::NotConnected,
            config_id,
        ));
        self.send_internal(
            &EdmAtCmdWrapper(ExecWifiStationAction {
                config_id,
                action: WifiStationAction::Activate,
            }),
            true,
        )?;

        // TODO: Await connected event?

        Ok(())
    }

    fn scan(&mut self) -> Result<Vec<WifiNetwork, 32>, WifiError> {
        match self.send_internal(&EdmAtCmdWrapper(WifiScan { ssid: None }), true) {
            Ok(resp) => resp
                .network_list
                .into_iter()
                .map(WifiNetwork::try_from)
                .collect(),
            Err(_) => Err(WifiError::UnexpectedResponse),
        }
    }

    fn is_connected(&self) -> bool {
        if !self.initialized {
            return false;
        }

        self.wifi_connection
            .as_ref()
            .map(|c| c.is_connected())
            .unwrap_or_default()
    }

    fn disconnect(&mut self) -> Result<(), WifiConnectionError> {
        if let Some(ref con) = self.wifi_connection {
            match con.wifi_state {
                WiFiState::Connected | WiFiState::NotConnected => {
                    // con.wifi_state = WiFiState::Inactive;
                    self.send_internal(
                        &EdmAtCmdWrapper(ExecWifiStationAction {
                            config_id: 0,
                            action: WifiStationAction::Deactivate,
                        }),
                        true,
                    )?;
                }
                WiFiState::Inactive => {}
            }
        } else {
            return Err(WifiConnectionError::FailedToDisconnect);
        }
        Ok(())
    }
}

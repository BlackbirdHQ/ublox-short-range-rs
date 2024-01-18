use core::str::FromStr;

use super::state::{self, LinkState};
use crate::{
    command::{
        edm::{urc::EdmEvent, SwitchToEdmCommand},
        general::SoftwareVersion,
        network::{
            responses::NetworkStatusResponse,
            types::{InterfaceType, NetworkStatus, NetworkStatusParameter},
            urc::{NetworkDown, NetworkUp},
            GetNetworkStatus,
        },
        system::{
            types::{BaudRate, ChangeAfterConfirm, EchoOn, FlowControl, Parity, StopBits},
            RebootDCE, SetEcho, SetRS232Settings, StoreCurrentConfig,
        },
        wifi::{
            types::DisconnectReason,
            urc::{WifiLinkConnected, WifiLinkDisconnected},
        },
        Urc,
    },
    connection::{WiFiState, WifiConnection},
    error::Error,
    network::WifiNetwork,
};
use atat::{asynch::AtatClient, UrcSubscription};
use embassy_time::{with_timeout, Duration, Timer};
use embedded_hal::digital::OutputPin;
use no_std_net::{Ipv4Addr, Ipv6Addr};

use super::AtHandle;

/// Background runner for the Ublox Module.
///
/// You must call `.run()` in a background task for the Ublox Module to operate.
pub struct Runner<
    'd,
    AT: AtatClient,
    RST: OutputPin,
    const MAX_CONNS: usize,
    const URC_CAPACITY: usize,
> {
    ch: state::Runner<'d>,
    at: AtHandle<'d, AT>,
    reset: RST,
    wifi_connection: Option<WifiConnection>,
    // connections: FnvIndexMap<PeerHandle, ConnectionType, MAX_CONNS>,
    urc_subscription: UrcSubscription<'d, EdmEvent, URC_CAPACITY, 2>,
}

impl<
        'd,
        AT: AtatClient,
        // AT: AtatClient + atat::UartExt,
        RST: OutputPin,
        const MAX_CONNS: usize,
        const URC_CAPACITY: usize,
    > Runner<'d, AT, RST, MAX_CONNS, URC_CAPACITY>
{
    pub(crate) fn new(
        ch: state::Runner<'d>,
        at: AtHandle<'d, AT>,
        reset: RST,
        urc_subscription: UrcSubscription<'d, EdmEvent, URC_CAPACITY, 2>,
    ) -> Self {
        Self {
            ch,
            at,
            reset,
            wifi_connection: None,
            urc_subscription,
            // connections: IndexMap::new(),
        }
    }

    pub(crate) async fn init(&mut self) -> Result<(), Error> {
        // Initilize a new ublox device to a known state (set RS232 settings)
        debug!("Initializing module");
        // Hard reset module
        self.reset().await?;

        // ## 2.2.6.1 AT request serial settings (EDM mode)
        //
        // The AT+UMRS command to change serial settings does not work exactly
        // the same as in command mode. When executed in the extended data mode,
        // it is not possible to change the settings directly using the
        // <change_after_confirm> parameter. Instead, the <change_after_confirm>
        // parameter must be set to 0 and the serial settings will take effect
        // when the module is reset.
        let baud_rate = BaudRate::B115200;
        self.at
            .send_edm(SetRS232Settings {
                baud_rate,
                flow_control: FlowControl::On,
                data_bits: 8,
                stop_bits: StopBits::One,
                parity: Parity::None,
                change_after_confirm: ChangeAfterConfirm::StoreAndReset,
            })
            .await?;

        self.restart(true).await?;

        self.at.send_edm(SoftwareVersion).await?;

        // Move to control
        // if let Some(size) = self.config.tls_in_buffer_size {
        //     self.at
        //         .send_edm(SetPeerConfiguration {
        //             parameter: PeerConfigParameter::TlsInBuffer(size),
        //         })
        //         .await?;
        // }

        // if let Some(size) = self.config.tls_out_buffer_size {
        //     self.at
        //         .send_edm(SetPeerConfiguration {
        //             parameter: PeerConfigParameter::TlsOutBuffer(size),
        //         })
        //         .await?;
        // }

        Ok(())
    }

    async fn wait_startup(&mut self, timeout: Duration) -> Result<(), Error> {
        let fut = async {
            loop {
                match self.urc_subscription.next_message_pure().await {
                    EdmEvent::ATEvent(Urc::StartUp) => return,
                    _ => {}
                }
            }
        };

        with_timeout(timeout, fut).await.map_err(|_| Error::Timeout)
    }

    pub async fn reset(&mut self) -> Result<(), Error> {
        warn!("Hard resetting Ublox Short Range");
        self.reset.set_low().ok();
        Timer::after(Duration::from_millis(100)).await;
        self.reset.set_high().ok();

        self.wait_startup(Duration::from_secs(4)).await?;

        self.enter_edm(Duration::from_secs(4)).await?;

        Ok(())
    }

    pub async fn restart(&mut self, store: bool) -> Result<(), Error> {
        warn!("Soft resetting Ublox Short Range");
        if store {
            self.at.send_edm(StoreCurrentConfig).await?;
        }

        self.at.send_edm(RebootDCE).await?;

        self.wait_startup(Duration::from_secs(10)).await?;

        info!("Module started again");
        self.enter_edm(Duration::from_secs(4)).await?;

        Ok(())
    }

    pub async fn enter_edm(&mut self, timeout: Duration) -> Result<(), Error> {
        info!("Entering EDM mode");

        // Switch to EDM on Init. If in EDM, fail and check with autosense
        let fut = async {
            loop {
                // Ignore AT results until we are successful in EDM mode
                if let Ok(_) = self.at.send(SwitchToEdmCommand).await {
                    // After executing the data mode command or the extended data
                    // mode command, a delay of 50 ms is required before start of
                    // data transmission.
                    Timer::after(Duration::from_millis(50)).await;
                    break;
                }
                Timer::after(Duration::from_millis(10)).await;
            }
        };

        with_timeout(timeout, fut)
            .await
            .map_err(|_| Error::Timeout)?;

        self.at.send_edm(SetEcho { on: EchoOn::On }).await?;

        Ok(())
    }

    pub async fn is_link_up(&mut self) -> Result<bool, Error> {
        // Determine link state
        let link_state = match self.wifi_connection {
            Some(ref conn)
                if conn.network_up && matches!(conn.wifi_state, WiFiState::Connected) =>
            {
                LinkState::Up
            }
            _ => LinkState::Down,
        };

        self.ch.set_link_state(link_state);

        Ok(link_state == LinkState::Up)
    }

    pub async fn run(mut self) -> ! {
        loop {
            let event = self.urc_subscription.next_message_pure().await;
            match event {
                EdmEvent::ATEvent(Urc::StartUp) => {
                    error!("AT startup event?! Device restarted unintentionally!");
                }
                EdmEvent::ATEvent(Urc::WifiLinkConnected(WifiLinkConnected {
                    connection_id: _,
                    bssid,
                    channel,
                })) => {
                    if let Some(ref mut con) = self.wifi_connection {
                        con.wifi_state = WiFiState::Connected;
                        con.network.bssid = bssid;
                        con.network.channel = channel;
                    } else {
                        debug!("[URC] Active network config discovered");
                        self.wifi_connection.replace(
                            WifiConnection::new(
                                WifiNetwork::new_station(bssid, channel),
                                WiFiState::Connected,
                                255,
                            )
                            .activate(),
                        );
                    }
                    self.is_link_up().await.unwrap();
                }
                EdmEvent::ATEvent(Urc::WifiLinkDisconnected(WifiLinkDisconnected {
                    reason,
                    ..
                })) => {
                    if let Some(ref mut con) = self.wifi_connection {
                        match reason {
                            DisconnectReason::NetworkDisabled => {
                                con.wifi_state = WiFiState::Inactive;
                            }
                            DisconnectReason::SecurityProblems => {
                                error!("Wifi Security Problems");
                            }
                            _ => {
                                con.wifi_state = WiFiState::NotConnected;
                            }
                        }
                    }

                    self.is_link_up().await.unwrap();
                }
                EdmEvent::ATEvent(Urc::WifiAPUp(_)) => todo!(),
                EdmEvent::ATEvent(Urc::WifiAPDown(_)) => todo!(),
                EdmEvent::ATEvent(Urc::WifiAPStationConnected(_)) => todo!(),
                EdmEvent::ATEvent(Urc::WifiAPStationDisconnected(_)) => todo!(),
                EdmEvent::ATEvent(Urc::EthernetLinkUp(_)) => todo!(),
                EdmEvent::ATEvent(Urc::EthernetLinkDown(_)) => todo!(),
                EdmEvent::ATEvent(Urc::NetworkUp(NetworkUp { interface_id })) => {
                    self.network_status_callback(interface_id).await.unwrap();
                }
                EdmEvent::ATEvent(Urc::NetworkDown(NetworkDown { interface_id })) => {
                    self.network_status_callback(interface_id).await.unwrap();
                }
                EdmEvent::ATEvent(Urc::NetworkError(_)) => todo!(),
                EdmEvent::StartUp => {
                    error!("EDM startup event?! Device restarted unintentionally!");
                }
                _ => {}
            };
        }
    }

    async fn network_status_callback(&mut self, interface_id: u8) -> Result<(), Error> {
        let NetworkStatusResponse {
            status: NetworkStatus::InterfaceType(InterfaceType::WifiStation),
            ..
        } = self
            .at
            .send_edm(GetNetworkStatus {
                interface_id,
                status: NetworkStatusParameter::InterfaceType,
            })
            .await?
        else {
            return Err(Error::Network);
        };

        let NetworkStatusResponse {
            status: NetworkStatus::Gateway(ipv4),
            ..
        } = self
            .at
            .send_edm(GetNetworkStatus {
                interface_id,
                status: NetworkStatusParameter::Gateway,
            })
            .await?
        else {
            return Err(Error::Network);
        };

        let ipv4_up = core::str::from_utf8(ipv4.as_slice())
            .ok()
            .and_then(|s| Ipv4Addr::from_str(s).ok())
            .map(|ip| !ip.is_unspecified())
            .unwrap_or_default();

        let NetworkStatusResponse {
            status: NetworkStatus::IPv6LinkLocalAddress(ipv6),
            ..
        } = self
            .at
            .send_edm(GetNetworkStatus {
                interface_id,
                status: NetworkStatusParameter::IPv6LinkLocalAddress,
            })
            .await?
        else {
            return Err(Error::Network);
        };

        let ipv6_up = core::str::from_utf8(ipv6.as_slice())
            .ok()
            .and_then(|s| Ipv6Addr::from_str(s).ok())
            .map(|ip| !ip.is_unspecified())
            .unwrap_or_default();

        // Use `ipv4_up` & `ipv6_up` to determine link state
        if let Some(ref mut con) = self.wifi_connection {
            con.network_up = ipv4_up && ipv6_up;
        }

        self.is_link_up().await?;

        Ok(())
    }
}
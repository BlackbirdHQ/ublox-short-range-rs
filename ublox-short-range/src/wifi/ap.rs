use crate::{
    client::UbloxClient,
    command::wifi::types::OperationMode,
    error::WifiHotspotError,
    wifi::{
        network::{WifiMode, WifiNetwork},
        options::{ConnectionOptions, HotspotOptions},
    },
    socket::Socket,
};
use atat::serde_at::CharVec;
use atat::AtatClient;
use embedded_time::Clock;

use heapless::{consts, ArrayLength};

pub trait WifiHotspot {
    /// Creates wireless hotspot service for host machine.
    fn create_hotspot(
        self,
        options: ConnectionOptions,
        configuration: HotspotOptions,
    ) -> Result<(), WifiHotspotError>;

    /// Stop serving a wireless network.
    ///
    /// **NOTE: All users connected will automatically be disconnected.**
    fn stop_hotspot(&mut self) -> Result<bool, WifiHotspotError>;
}

impl<C, CLK, N, L> WifiHotspot for UbloxClient<C, CLK, N, L>
where
    C: AtatClient,
    CLK: Clock,
    N: ArrayLength<Option<Socket<L, CLK>>>,
    L: ArrayLength<u8>,
{
    /// Creates wireless hotspot service for host machine.
    fn create_hotspot(
        self,
        options: ConnectionOptions,
        configuration: HotspotOptions,
    ) -> Result<(), WifiHotspotError> {
        let _network = WifiNetwork {
            bssid: CharVec::<consts::U20>::new(),
            op_mode: OperationMode::AdHoc,
            ssid: options.ssid,
            channel: configuration.channel.unwrap() as u8,
            rssi: 1,
            authentication_suites: 0,
            unicast_ciphers: 0,
            group_ciphers: 0,
            mode: WifiMode::AccessPoint,
        };
        // self.wifi_connection.set(Some(WifiConnection::new(network)));
        Ok(())
    }

    /// Stop serving a wireless network.
    ///
    /// **NOTE: All users connected will automatically be disconnected.**
    fn stop_hotspot(&mut self) -> Result<bool, WifiHotspotError> {
        Ok(true)
    }
}

//! Unsolicited responses for WiFi Commands
use super::types::*;
use atat::atat_derive::AtatResp;
use atat::heapless_bytes::Bytes;

/// 7.15 Wi-Fi Link connected +UUWLE
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct WifiLinkConnected {
    /// Not the config id!
    /// Shame Ublox!
    /// UBX-14044127
    #[at_arg(position = 0)]
    pub connection_id: u32,
    #[at_arg(position = 1)]
    pub bssid: Bytes<20>,
    #[at_arg(position = 2)]
    pub channel: u8,
}

/// 7.16 Wi-Fi Link disconnected +UUWLD
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct WifiLinkDisconnected {
    #[at_arg(position = 0)]
    pub connection_id: u32,
    #[at_arg(position = 1)]
    pub reason: DisconnectReason,
}

/// 7.17 Wi-Fi Access point up +UUWAPU
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct WifiAPUp {
    #[at_arg(position = 0)]
    pub connection_id: u32,
}

/// 7.18 Wi-Fi Access point down +UUWAPD
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct WifiAPDown {
    #[at_arg(position = 0)]
    pub connection_id: u32,
}

/// 7.19 Wi-Fi Access point station connected +UUWAPSTAC
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct WifiAPStationConnected {
    #[at_arg(position = 0)]
    pub station_id: u32,
    #[at_arg(position = 1)]
    pub mac_addr: Bytes<20>,
}

/// 7.20 Wi-Fi Access point station disconnected +UUWAPSTAD
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct WifiAPStationDisconnected {
    #[at_arg(position = 0)]
    pub station_id: u32,
}

//! Unsolicited responses for WiFi Commands
use crate::socket::SocketHandle;
use atat::atat_derive::AtatResp;
use heapless::{consts, String};
use super::types::*;
use no_std_net::IpAddr;

/// 7.15 Wi-Fi Link connected +UUWLE
#[derive(Clone, AtatResp)]
pub struct WifiLinkConnected {
    #[at_arg(position = 0)]
    pub connection_id: u32,
    #[at_arg(position = 1)]
    pub bssid: String<consts::U64>,
    #[at_arg(position = 2)]
    pub channel: u8,
}

/// 7.16 Wi-Fi Link disconnected +UUWLD
#[derive(Clone, AtatResp)]
pub struct WifiLinkDisconnected {
    #[at_arg(position = 0)]
    pub connection_id: u32,
    #[at_arg(position = 1)]
    pub reason: DisconnectReason,
}

/// 7.17 Wi-Fi Access point up +UUWAPU
#[derive(Clone, AtatResp)]
pub struct WifiAPUp {
    #[at_arg(position = 0)]
    pub connection_id: u32,
}

/// 7.18 Wi-Fi Access point down +UUWAPD
#[derive(Clone, AtatResp)]
pub struct WifiAPDown {
    #[at_arg(position = 0)]
    pub connection_id: u32,
}

/// 7.19 Wi-Fi Access point station connected +UUWAPSTAC
#[derive(Clone, AtatResp)]
pub struct WifiAPStationConnected {
    #[at_arg(position = 0)]
    pub station_id: u32,
    #[at_arg(position = 1)]
    pub mac_addr: String<consts::U20>,
}

/// 7.20 Wi-Fi Access point station disconnected +UUWAPSTAD
#[derive(Clone, AtatResp)]
pub struct WifiAPStationDisconnected {
    #[at_arg(position = 0)]
    pub station_id: u32,
}
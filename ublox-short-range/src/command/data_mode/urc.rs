//! Unsolicited responses for Data mode Commands
use super::types::*;
use atat::atat_derive::AtatResp;
use atat::serde_at::CharVec;

/// 5.10 Peer connected +UUDPC
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct PeerConnected {
    #[at_arg(position = 0)]
    pub handle: usize,
    #[at_arg(position = 1)]
    pub connection_type: ConnectionType,
    #[at_arg(position = 2)]
    pub protocol: IPProtocol,
    // #[at_arg(position = 3)]
    // pub local_address: IpAddr,
    #[at_arg(position = 3)]
    pub local_address: CharVec<40>,
    #[at_arg(position = 4)]
    pub local_port: u16,
    // #[at_arg(position = 5)]
    // pub remote_address: IpAddr,
    #[at_arg(position = 5)]
    pub remote_address: CharVec<40>,
    #[at_arg(position = 6)]
    pub remote_port: u16,
}

/// 5.11 Peer disconnected +UUDPD
#[derive(Debug, PartialEq, Clone, AtatResp)]
pub struct PeerDisconnected {
    #[at_arg(position = 0)]
    pub handle: usize,
}

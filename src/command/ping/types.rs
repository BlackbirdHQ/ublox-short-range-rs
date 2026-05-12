//! Argument and parameter types used by Ping Commands and Responses

/// Indicates the number of iterations for the ping command.
/// - Range: 1-2147483647
/// - Default value: 4
// pub type RetryNum = (u32, Option<PacketSize>);
/// Size in bytes of the echo packet payload.
/// - Range: 4-1472
/// - Default value: 32
// pub type PacketSize = (u16, Option<Timeout>);
/// The maximum time in milliseconds to wait for an echo reply response.
/// - Range: 10-60000
/// - Default value: 5000
// pub type Timeout = (u16, Option<TTL>);
/// The value of TTL to be set for the outgoing echo request packet. In the URC, it
/// provides the TTL value received in the incoming packet.
/// - Range: 1-255
/// - Default value: 32
// pub type TTL = (u8, Option<Interval>);
/// The time in milliseconds to wait after an echo reply response before sending the next
/// echo request.
/// - Range: 0-60000
/// - Default value: 1000
// pub type Interval = u16;

/// Error code reported in the `+UUPINGER` URC.
///
/// Per the ODIN-W2 AT command manual only three codes carry well-defined
/// semantics; every other value is an internal module error.
///
/// `Other` is also used as a Rust-side fallback when sending the AT command
/// itself fails before any URC is received.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PingError {
    /// 3: Timeout — the remote host did not respond within the timeout.
    ///
    /// Also used when `+UUPING` reports `rtt = -1`.
    Timeout,
    /// 8: Could not resolve remote host (DNS lookup failed).
    CannotResolveHost,
    /// 17: Network not available (no Wi-Fi connection established).
    NetworkNotAvailable,
    /// Any other code reported by the module, or a Rust-side fallback used
    /// when the `+UPING` AT command fails before any URC is received.
    Other,
}

impl From<u8> for PingError {
    fn from(value: u8) -> Self {
        match value {
            3 => Self::Timeout,
            8 => Self::CannotResolveHost,
            17 => Self::NetworkNotAvailable,
            _ => Self::Other,
        }
    }
}

impl<'de> atat::serde_at::serde::Deserialize<'de> for PingError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: atat::serde_at::serde::Deserializer<'de>,
    {
        <u8 as atat::serde_at::serde::Deserialize>::deserialize(deserializer).map(Self::from)
    }
}

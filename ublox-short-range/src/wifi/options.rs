use embedded_nal::Ipv4Addr;
use heapless::String;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
/// Channel to broadcast wireless hotspot on.
pub enum Channel {
    /// Channel 1
    One = 1,
    /// Channel 2
    Two = 2,
    /// Channel 3
    Three = 3,
    /// Channel 4
    Four = 4,
    /// Channel 5
    Five = 5,
    /// Channel 6
    Six = 6,
}

#[allow(dead_code)]
#[derive(Debug)]
/// Band type of wireless hotspot.
pub enum Band {
    /// Band `A`
    A,
    /// Band `BG`
    Bg,
}

#[derive(Debug, Default)]
pub struct HotspotOptions {
    pub(crate) channel: Option<Channel>,
    pub(crate) band: Option<Band>,
}

impl HotspotOptions {
    pub fn new() -> Self {
        Self {
            channel: Some(Channel::One),
            band: Some(Band::Bg),
        }
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn band(mut self, band: Band) -> Self {
        self.band = Some(band);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, defmt::Format)]
pub struct ConnectionOptions {
    pub ssid: String<64>,
    pub password: Option<String<64>>,

    #[defmt(Debug2Format)]
    pub ip: Option<Ipv4Addr>,
    #[defmt(Debug2Format)]
    pub subnet: Option<Ipv4Addr>,
    #[defmt(Debug2Format)]
    pub gateway: Option<Ipv4Addr>,
}

impl ConnectionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ssid(mut self, ssid: String<64>) -> Self {
        self.ssid = ssid;
        self
    }

    pub fn password(mut self, password: String<64>) -> Self {
        self.password = Some(password);
        self
    }

    pub fn ip_address(mut self, ip_addr: Ipv4Addr) -> Self {
        self.ip = Some(ip_addr);
        self.subnet = if let Some(subnet) = self.subnet {
            Some(subnet)
        } else {
            Some(Ipv4Addr::new(255, 255, 255, 0))
        };

        self.gateway = if let Some(gateway) = self.gateway {
            Some(gateway)
        } else {
            Some(Ipv4Addr::new(192, 168, 2, 1))
        };
        self
    }

    pub fn subnet_address(mut self, subnet_addr: Ipv4Addr) -> Self {
        self.subnet = Some(subnet_addr);

        self.ip = if let Some(ip) = self.ip {
            Some(ip)
        } else {
            Some(Ipv4Addr::new(192, 168, 2, 1))
        };

        self.gateway = if let Some(gateway) = self.gateway {
            Some(gateway)
        } else {
            Some(Ipv4Addr::new(192, 168, 2, 1))
        };

        self
    }

    pub fn gateway_address(mut self, gateway_addr: Ipv4Addr) -> Self {
        self.gateway = Some(gateway_addr);

        self.subnet = if let Some(subnet) = self.subnet {
            Some(subnet)
        } else {
            Some(Ipv4Addr::new(255, 255, 255, 0))
        };

        self.ip = if let Some(ip) = self.ip {
            Some(ip)
        } else {
            Some(Ipv4Addr::new(192, 168, 2, 1))
        };
        self
    }
}

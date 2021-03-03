//! Responses for General Commands
use atat::atat_derive::AtatResp;
use heapless::{consts, String};

/// 3.2 Manufacturer identification +CGMI
#[derive(Clone, AtatResp)]
pub struct ManufacturerIdentificationResponse {
    /// Text string that identifies the Manufacture identification.
    #[at_arg(position = 0)]
    pub manufacturer_id: String<consts::U64>,
}

/// 3.3 Model identification +CGMI
#[derive(Clone, AtatResp)]
pub struct ModelIdentificationResponse {
    /// Text string that identifies the Model identification.
    #[at_arg(position = 0)]
    pub model: String<consts::U64>,
}

/// 3.3 Model identification +CGMI
#[derive(Clone, AtatResp)]
pub struct SoftwareVersionResponse {
    /// Text string that identifies the Model identification.
    #[at_arg(position = 0)]
    pub version: String<consts::U64>,
}

/// 3.5 Serial number +CGSN
#[derive(Clone, AtatResp)]
pub struct SerialNumberResponse {
    /// Text string that identifies the serial number.
    #[at_arg(position = 0)]
    pub serial_number: String<consts::U64>,
}

/// 3.10 Identification information I0
#[derive(Clone, AtatResp)]
pub struct IdentificationInfomationTypeCodeResponse {
    /// Text string that identifies the serial number.
    #[at_arg(position = 0)]
    pub serial_number: String<consts::U64>,
}

/// 3.10 Identification information I9
#[derive(Clone, AtatResp)]
pub struct IdentificationInfomationSoftwareVersionResponse {
    /// Text string that identifies the serial number.
    #[at_arg(position = 0)]
    pub serial_number: String<consts::U64>,
}

/// 3.10 Identification information I10
#[derive(Clone, AtatResp)]
pub struct IdentificationInfomationMCUIDResponse {
    /// Text string that identifies the serial number.
    #[at_arg(position = 0)]
    pub serial_number: String<consts::U64>,
}

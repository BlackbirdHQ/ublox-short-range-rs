//! ### 3 - General
pub mod responses;
pub mod types;

use atat::atat_derive::AtatCmd;
use responses::*;
use types::*;

use super::NoResponse;

/// 3.2 Manufacturer identification +CGMI
///
/// Text string that identifies the manufacturer.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGMI", ManufacturerIdentificationResponse, timeout_ms = 1000)]
pub struct ManufacturerIdentification;

/// 3.3 Model identification +CGMM
///
/// Text string that identifies the model.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGMM", ModelIdentificationResponse, timeout_ms = 1000)]
pub struct ModelIdentification;

/// 3.4 Software version identification +CGMR
///
/// Text string that identifies the software version.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGMR", SoftwareVersionResponse, timeout_ms = 1000)]
pub struct SoftwareVersion;

/// 3.5 Serial number +CGSN
///
/// The product serial number.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CGSN", SerialNumberResponse, timeout_ms = 1000)]
pub struct SerialNumber;

/// 3.6 Manufacturer identification +GMI
///
/// Text string that identifies the manufacturer.
#[derive(Clone, AtatCmd)]
#[at_cmd("+GMI", ManufacturerIdentificationResponse, timeout_ms = 1000)]
pub struct ManufacturerIdentification2;

/// 3.7 Software version identification +CGMR
///
/// Text string that identifies the software version.
#[derive(Clone, AtatCmd)]
#[at_cmd("+GMR", SoftwareVersionResponse, timeout_ms = 1000)]
pub struct SoftwareVersion2;

/// 3.8 Serial number +CGSN
///
/// The product serial number.
#[derive(Clone, AtatCmd)]
#[at_cmd("+GSN", SerialNumberResponse, timeout_ms = 1000)]
pub struct SerialNumber2;

/// 3.9 Identification information I0
///
/// Identificationinformation.
#[derive(Clone, AtatCmd)]
#[at_cmd("I0", IdentificationInformationTypeCodeResponse, timeout_ms = 1000)]
pub struct IdentificationInformationTypeCode;

/// 3.9 Identification information I9
///
/// Identificationinformation.
#[derive(Clone, AtatCmd)]
#[at_cmd(
    "I9",
    IdentificationInformationSoftwareVersionResponse,
    timeout_ms = 1000
)]
pub struct IdentificationInformationSoftwareVersion;

/// 3.9 Identification information I10
///
/// Identificationinformation.
#[derive(Clone, AtatCmd)]
#[at_cmd("I10", IdentificationInformationMCUIDResponse, timeout_ms = 1000)]
pub struct IdentificationInformationMCUID;

/// 3.11 Set greeting text +CSGT
///
/// Sets the greeting text. Max 48 characters.
/// Configures and activates/deactivates the greeting text. The configuration change
/// in the greeting text will be applied at the subsequent boot. If active, the greeting
/// text is shown at boot once, on any AT interface, if the module start up mode is set to
/// command mode.
#[derive(Clone, AtatCmd)]
#[at_cmd("+CSGT", NoResponse, timeout_ms = 1000)]
pub struct SetGreetingText<'a> {
    #[at_arg(position = 0, len = 48)]
    pub mode: GreetingTextMode<'a>,
}

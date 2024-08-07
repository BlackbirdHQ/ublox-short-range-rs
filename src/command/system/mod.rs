//! ### 20 - System Commands
pub mod responses;
pub mod types;

use atat::atat_derive::AtatCmd;
use responses::*;
use types::*;

use super::NoResponse;

/// 4.1 Store current configuration &W
///
/// Commits all the settings to be stored in start up database. The parameters are
///  written to non-volatile memory when +CPWROFF is issued.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("&W0", NoResponse, timeout_ms = 1000)]
pub struct StoreCurrentConfig;

/// 4.2 Set to default configuration Z
///
/// Resets the profile to the last stored configuration. Any settings committed with
/// AT&W will be discarded. The restored settings will be used after a reboot.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("Z0", NoResponse, timeout_ms = 1000)]
pub struct SetToDefaultConfig;

/// 4.3 Set to factory defined configuration +UFACTORY
///
/// Reset to factory defined defaults. A reboot is required before using the new settings.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UFACTORY", NoResponse, timeout_ms = 1000)]
pub struct ResetToFactoryDefaults;

/// 4.4 Circuit 108/2 (DTR) behavior &D
///
/// Controls the behaviour of RS232 circuit 108/2 - Data Terminal Ready (DTR) - on
/// changes between ASSERTED (logical 0 on UART_DSR signal) and DEASSERTED
/// (logical 1 on UART_DSR signal) states.
/// The DTR line is connected to the DSR pin on the module.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("&D", NoResponse, timeout_ms = 1000)]
pub struct SetDTRBehavior {
    #[at_arg(position = 0)]
    pub mode: DTRMode,
}

/// 4.5 DSR Override &S
///
/// Selects how the module will control RS232 circuit 107 - Data Set Ready (DSR)
/// between ASSERTED (logical 0 on signal UART_DTR) and DEASSERTED (logical 1 on
/// signal UART_DTR) states.
/// The DSR line is connected to the DTR pin on the module.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("&S", NoResponse, timeout_ms = 1000)]
pub struct SetDSROverride {
    #[at_arg(position = 0)]
    pub mode: DSRAssertMode,
}

/// 4.6 Echo On/Off E
///
/// This command configures whether or not the unit echoes the characters received
/// from the DTE in Command Mode. If <echo_on> is omitted, it turns off the echoing.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("E", NoResponse, timeout_ms = 1000, value_sep = false)]
pub struct SetEcho {
    #[at_arg(position = 0)]
    pub on: EchoOn,
}

/// 4.7 Escape character S2
///
/// The escape sequence is the sequence that forces the module to switch from the
/// data mode to command mode, or to enter configuration mode over the air. To enter
/// configuration mode over the air, this must be enabled on the specific server or peer,
/// and all three escape characters must be transmitted in a single frame.
/// Upon successful transition to the command mode, the DCE will transmit an OK
/// response.
/// Factory default: 43, the "+" character.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("S2", NoResponse, timeout_ms = 1000)]
pub struct SetEscapeCharacter {
    #[at_arg(position = 0)]
    pub esc_char: u8,
}

/// 4.8 Command line termination character S3
///
/// Writes command line termination character.
/// This setting changes the decimal value of the character recognized by the DCE from
/// the DTE to terminate an incoming command line. It is also generated by the DCE as
/// part of the header, trailer, and terminator for result codes and information text along
/// with the S4 parameter.
/// The previous value of S3 is used to determine the command line termination
/// character for entry of the command line containing the S3 setting command.
/// However, the result code issued shall use the value of S3 as set during the processing
/// of the command line. For example, if S3 was previously set to 13 and the command
/// line "ATS3=30" is issued, the command line shall be terminated with a CR, character
/// (13), but the result code issued will use the character with the ordinal value 30 instead
/// of the CR.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("S3", NoResponse, timeout_ms = 1000)]
pub struct SetLineTerminationCharacter {
    /// 0...127  Factory default: 13
    #[at_arg(position = 0)]
    pub line_term: u8,
}

/// 4.9 Response formatting character S4
///
/// Writes response formatting character.
/// This setting changes the decimal value of the character generated by the DCE as part
/// of the header, trailer, and terminator for result codes and information text, along with
/// the S3 parameter.
/// If the value of S4 is changed in a command line, the result code issued in response to
/// that command line will use the new value of S4.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("S4", NoResponse, timeout_ms = 1000)]
pub struct SetResponseFormattingCharacter {
    /// 0...127  Factory default: 10
    #[at_arg(position = 0)]
    pub term: u8,
}

/// 4.10 Backspace character S5
///
/// Writes backspace character.
/// This setting changes the decimal value of the character recognized by the DCE as a
/// request to delete from the command line, the immediately preceding character.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("S5", NoResponse, timeout_ms = 1000)]
pub struct SetBackspaceCharacter {
    /// 0...127  Factory default: 8
    #[at_arg(position = 0)]
    pub backspace: u8,
}

/// 4.11 Software update +UFWUPD
///
/// Force start of the boot loader. The boot loader will start at the defined baud rate.
/// To update any binary image other than the u-connect software, enter the bootloader
/// mode and follow the boot menu commands.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UFWUPD", SoftwareUpdateResponse, timeout_ms = 1000)]
pub struct SoftwareUpdate {
    #[at_arg(position = 0)]
    pub mode: SoftwareUpdateMode,
    #[at_arg(position = 1)]
    pub baud: SoftwareUpdateBaudRate,
}

/// 4.12 Module switch off +CPWROFF
///
/// Reboot the DCE. During shutdown, the settings marked for storing to start up the
/// database by &W are written in the non-volatile memory of the module.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+CPWROFF", NoResponse, timeout_ms = 1000)]
pub struct RebootDCE;

/// 4.13 Module start mode +UMSM
///
/// Writes start mode
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UMSM", NoResponse, timeout_ms = 1000)]
pub struct ModuleStart {
    #[at_arg(position = 0)]
    pub mode: ModuleStartMode,
}

/// 4.14 Set Local address +UMLA
///
/// Sets the local address of the interface id. A DCE reboot is required before an address
/// change takes effect.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UMLA", NoResponse, timeout_ms = 1000)]
pub struct SetLocalAddress<'a> {
    #[at_arg(position = 0)]
    pub interface_id: InterfaceID,
    /// MAC address of the interface id. If the address is set to 000000000000, the local
    /// address will be restored to factory-programmed value.
    /// The least significant bit of the first octet of the <address> must be 0; that is, the
    /// <address> must be a unicast address.
    #[at_arg(position = 1, len = 12)]
    pub mac_address: &'a atat::serde_bytes::Bytes,
}

/// 4.14 Get Local address +UMLA
///
/// Reads the local address of the interface id.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UMLA", LocalAddressResponse, timeout_ms = 1000)]
pub struct GetLocalAddress {
    #[at_arg(position = 0)]
    pub interface_id: InterfaceID,
}

/// 4.15 System status +UMSTAT
///
/// Reads current status of the system. If <status_id> is omitted, all applicable ids will be
/// listed.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UMSTAT", SystemStatusResponse, timeout_ms = 1000)]
pub struct SystemStatus {
    #[at_arg(position = 0)]
    pub status_id: StatusID,
}

/// 4.16 RS232 Settings +UMRS
///
/// After receiving the OK response, the DTE shall wait for at least 40 ms for ODIN-
/// W2 and 1 second for NINA-B1, NINA-B3, and ANNA-B112 before issuing a new AT
/// command, to guarantee a proper baudrate reconfiguration.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UMRS", NoResponse, timeout_ms = 1000)]
pub struct SetRS232Settings {
    #[at_arg(position = 0)]
    pub baud_rate: BaudRate,
    #[at_arg(position = 1)]
    pub flow_control: FlowControl,
    #[at_arg(position = 2)]
    pub data_bits: u8,
    #[at_arg(position = 3)]
    pub stop_bits: StopBits,
    #[at_arg(position = 4)]
    pub parity: Parity,
    #[at_arg(position = 5)]
    pub change_after_confirm: ChangeAfterConfirm,
}

/// 4.17 Route radio signals to GPIOs +UMRSIG
/// Enable routing of radio signals to EXT_TX_EN and EXT_RX_EN pins.
/// When routing is enabled on both the pins, it is recommended not to use other
/// GPIO commands on the same pins to avoid undefined behavior.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UMRSIG", NoResponse, timeout_ms = 1000)]
pub struct SetRouteSignalsGPIO {
    #[at_arg(position = 0)]
    pub mode: Mode,
}

/// 4.18 Power regulator +UPWRREG
///
/// Enable/disable automatic switch between DC/DC and LDO power regulators.
/// For the settings to take effect, use the commands - &W and +CPWROFF to store the configuration to
/// start up database and reboot the module.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UPWRREG", NoResponse, timeout_ms = 1000)]
pub struct SetPowerRegulatorSettings {
    #[at_arg(position = 0)]
    pub settings: PowerRegulatorSettings,
}

/// 4.19 LPO detection +UMLPO
///
/// Checks if Low Power Oscillator (LPO) is detected or not.
#[derive(Debug, PartialEq, Clone, AtatCmd)]
#[at_cmd("+UMLPO?", LPODetectionResponse, timeout_ms = 1000)]
pub struct GetLPODetection;

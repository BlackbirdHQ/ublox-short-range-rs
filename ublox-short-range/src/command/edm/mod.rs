// pub mod responses;
pub mod types;
pub mod urc;

use crate::command::{data_mode, data_mode::ChangeMode};
use crate::command::{NoResponse, Urc};
/// Containing EDM structs with custom serialaization and deserilaisation.
use atat::AtatCmd;
use heapless::{consts, ArrayLength, Vec};
use types::*;

#[inline]
pub(crate) fn calc_payload_len(resp: &[u8]) -> usize {
    ((((resp[1] as u16) << 8) + resp[2] as u16) & EDM_FULL_SIZE_FILTER) as usize
}
/// EDM wrapper for AT-Commands
// Note:
// The AT+UMRS command to change serial settings does not work exactly the same as in command
// mode. When executed in the extended data mode, it is not possible to change the settings directly
// using the <change_after_confirm> parameter. Instead the <change_after_confirm> parameter must
// be set to 0 and the serial settings will take effect when the module is reset.
#[derive(Debug, Clone)]
pub(crate) struct EdmAtCmdWrapper<T: AtatCmd>(pub T);

impl<T> atat::AtatCmd for EdmAtCmdWrapper<T>
where
    T: AtatCmd,
    <T as atat::AtatCmd>::CommandLen: core::ops::Add<EdmAtCmdOverhead>,
    <<T as atat::AtatCmd>::CommandLen as core::ops::Add<EdmAtCmdOverhead>>::Output: ArrayLength<u8>,
{
    type Response = T::Response;
    type CommandLen =
        <<T as atat::AtatCmd>::CommandLen as core::ops::Add<EdmAtCmdOverhead>>::Output;

    fn as_bytes(&self) -> Vec<u8, Self::CommandLen> {
        let mut s: Vec<u8, Self::CommandLen> = Vec::new();
        let at_vec = self.0.as_bytes();
        let payload_len = (at_vec.len() + 2) as u16;
        s.extend(
            [
                STARTBYTE,
                (payload_len >> 8) as u8 & EDM_SIZE_FILTER,
                (payload_len & 0xffu16) as u8,
                0x00,
                PayloadType::ATRequest as u8,
            ]
            .iter(),
        );
        s.extend(at_vec.iter());
        s.push(ENDBYTE).unwrap_or_else(|_| core::unreachable!());
        return s;
    }

    fn parse(&self, resp: &[u8]) -> core::result::Result<Self::Response, atat::Error> {
        if resp.len() < PAYLOAD_OVERHEAD
            || !resp.starts_with(&[STARTBYTE])
            || !resp.ends_with(&[ENDBYTE])
        {
            return Err(atat::Error::InvalidResponse);
        };
        let payload_len = calc_payload_len(resp);
        if resp.len() != payload_len + EDM_OVERHEAD || resp[4] != PayloadType::ATConfirmation as u8
        {
            return Err(atat::Error::InvalidResponse);
        }
        // Isolate the AT_response
        let mut at_resp = &resp[AT_COMMAND_POSITION..PAYLOAD_POSITION + payload_len];

        //Recieved OK response code in EDM response?
        if let Some(pos) = resp
            .windows(b"\r\nOK".len())
            .position(|window| window == b"\r\nOK")
        {
            //Cutting OK out leaves an empth string for NoResponse, for other responses just removes "\r\nOK\r\n"
            at_resp = &resp[AT_COMMAND_POSITION..pos];
        }

        self.0.parse(at_resp)
    }

    fn force_receive_state(&self) -> bool {
        true
    }

    fn max_timeout_ms(&self) -> u32 {
        self.0.max_timeout_ms()
    }
}

#[derive(Debug, Clone)]
pub struct EdmDataCommand<'a> {
    pub channel: u8,
    pub data: &'a [u8],
}

impl<'a> atat::AtatCmd for EdmDataCommand<'a> {
    type Response = NoResponse;
    type CommandLen = <DataPackageSize as core::ops::Add<consts::U4>>::Output;

    fn as_bytes(&self) -> Vec<u8, Self::CommandLen> {
        let mut s: Vec<u8, Self::CommandLen> = Vec::new();
        let payload_len = (self.data.len() + 3) as u16;
        s.extend(
            [
                STARTBYTE,
                (payload_len >> 8) as u8 & EDM_SIZE_FILTER,
                (payload_len & 0xffu16) as u8,
                0x00,
                PayloadType::DataCommand as u8,
                self.channel,
            ]
            .iter(),
        );
        s.extend(self.data);
        s.push(ENDBYTE).unwrap_or_else(|_| core::unreachable!());
        return s;
    }

    fn parse(&self, _resp: &[u8]) -> core::result::Result<Self::Response, atat::Error> {
        Ok(NoResponse)
    }

    fn max_timeout_ms(&self) -> u32 {
        10000
    }

    fn expects_response_code(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct EdmResendConnectEventsCommand;

impl atat::AtatCmd for EdmResendConnectEventsCommand {
    type Response = NoResponse;
    type CommandLen = consts::U8;

    fn as_bytes(&self) -> Vec<u8, Self::CommandLen> {
        let mut s: Vec<u8, Self::CommandLen> = Vec::new();
        s.extend(
            [
                STARTBYTE,
                0x00,
                0x02,
                0x00,
                PayloadType::ResendConnectEventsCommand as u8,
                ENDBYTE,
            ]
            .iter(),
        );
        return s;
    }

    fn parse(&self, _resp: &[u8]) -> core::result::Result<Self::Response, atat::Error> {
        Ok(NoResponse)
    }

    fn max_timeout_ms(&self) -> u32 {
        10000
    }

    fn expects_response_code(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
pub struct SwitchToEdmCommand;

impl atat::AtatCmd for SwitchToEdmCommand {
    type Response = NoResponse;
    type CommandLen = <ChangeMode as atat::AtatCmd>::CommandLen;

    fn as_bytes(&self) -> Vec<u8, Self::CommandLen> {
        return ChangeMode {
            mode: data_mode::types::Mode::ExtendedDataMode,
        }
        .as_bytes();
    }

    fn parse(&self, resp: &[u8]) -> core::result::Result<Self::Response, atat::Error> {
        // Parse EDM startup command
        let correct = &[0xAAu8, 0x00, 0x02, 0x00, 0x71, 0x55]; // &[0xAAu8,0x00,0x06,0x00,0x45,0x4f,0x4b,0x0D,0x0a,0x55]; //AA 00 06 00 44 41 54 0D 0A 0D 0A 4F 4B 0D 0A 55 ?
        if resp.len() != correct.len() {
            return Err(atat::Error::InvalidResponse);
        } else if resp
            .windows(correct.len())
            .position(|window| window == correct)
            != Some(0)
        {
            //TODO: check this
            return Err(atat::Error::InvalidResponse);
        }
        Ok(NoResponse)
    }

    fn force_receive_state(&self) -> bool {
        true
    }

    fn max_timeout_ms(&self) -> u32 {
        ChangeMode {
            mode: data_mode::types::Mode::ExtendedDataMode,
        }.max_timeout_ms()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::command::{
        system::{responses::SystemStatusResponse, types::StatusID, SystemStatus},
        AT,
    };
    use atat::{
        heapless::{consts, Vec},
        AtatCmd, Error,
    };

    #[test]
    fn parse_at_commands() {
        let parse = EdmAtCmdWrapper(AT);
        let correct_response = NoResponse;

        // AT-command: "AT"
        let correct_cmd = Vec::<u8, consts::U10>::from_slice(&[
            0xAAu8, 0x00, 0x06, 0x00, 0x44, 0x41, 0x54, 0x0D, 0x0a, 0x55,
        ])
        .unwrap();
        // AT-response: NoResponse
        let response = &[
            0xAAu8,
            0x00,
            0x02,
            0x00,
            PayloadType::ATConfirmation as u8,
            0x55,
        ];
        assert_eq!(parse.as_bytes(), correct_cmd);
        assert_eq!(parse.parse(response), Ok(correct_response));

        let parse = EdmAtCmdWrapper(SystemStatus {
            status_id: StatusID::SavedStatus,
        });
        let correct_response = SystemStatusResponse {
            status_id: StatusID::SavedStatus,
            status_val: 100,
        };
        // AT-command: "at+umstat=1"
        let correct = Vec::<u8, consts::U19>::from_slice(&[
            0xAAu8, 0x00, 0x0F, 0x00, 0x44, 0x41, 0x54, 0x2b, 0x55, 0x4d, 0x53, 0x54, 0x41, 0x54,
            0x3d, 0x31, 0x0D, 0x0A, 0x55,
        ])
        .unwrap();
        // AT-response: "at+umstat:1,100"
        let response = &[
            0xAAu8,
            0x00,
            0x11,
            0x00,
            PayloadType::ATConfirmation as u8,
            0x2B,
            0x55,
            0x4D,
            0x53,
            0x54,
            0x41,
            0x54,
            0x3A,
            0x31,
            0x2C,
            0x31,
            0x30,
            0x30,
            0x0D,
            0x0A,
            0x55,
        ];
        assert_eq!(parse.as_bytes(), correct);
        assert_eq!(parse.parse(response), Ok(correct_response));
    }

    #[test]
    fn parse_wrong_at_responses() {
        let parse = EdmAtCmdWrapper(AT);
        // AT-response: NoResponse
        let response = &[
            0xAAu8,
            0x00,
            0x06,
            0x00,
            PayloadType::ATConfirmation as u8,
            0x55,
        ];
        assert_eq!(
            parse.parse(response),
            Err(Error::InvalidResponse),
            "Response shorter than indicated not invalid"
        );

        let parse = EdmAtCmdWrapper(SystemStatus {
            status_id: StatusID::SavedStatus,
        });
        // AT-response: "at+umstat:1,100"
        let response = &[
            0xAAu8,
            0x00,
            0x01,
            0x00,
            PayloadType::ATConfirmation as u8,
            0x2B,
            0x55,
            0x4D,
            0x53,
            0x54,
            0x41,
            0x54,
            0x3A,
            0x31,
            0x2C,
            0x31,
            0x30,
            0x30,
            0x0D,
            0x0A,
            0x55,
        ];
        assert_eq!(
            parse.parse(response),
            Err(Error::InvalidResponse),
            "Response longer than indicated not invalid"
        );

        let response = &[
            0xAAu8,
            0x00,
            0x11,
            0x00,
            PayloadType::ATConfirmation as u8,
            0x2B,
            0x55,
            0x4D,
            0x53,
            0x54,
            0x41,
            0x54,
            0x3A,
            0x31,
            0x2C,
            0x31,
            0x30,
            0x30,
            0x0D,
            0x0A,
            0x00,
        ];
        assert_eq!(
            parse.parse(response),
            Err(Error::InvalidResponse),
            "Response wrong endbyte not invalid"
        );

        let response = &[
            0x00u8,
            0x00,
            0x11,
            0x00,
            PayloadType::ATConfirmation as u8,
            0x2B,
            0x55,
            0x4D,
            0x53,
            0x54,
            0x41,
            0x54,
            0x3A,
            0x31,
            0x2C,
            0x31,
            0x30,
            0x30,
            0x0D,
            0x0A,
            0x55,
        ];
        assert_eq!(
            parse.parse(response),
            Err(Error::InvalidResponse),
            "Response wrong startbyte not invalid"
        );

        let response = &[
            0xAAu8,
            0x00,
            0x02,
            0x00,
            PayloadType::ATConfirmation as u8,
            0x55,
        ];
        assert_eq!(
            parse.parse(response),
            Err(Error::ParseString),
            "Response wrong not invalid"
        );
    }

    #[test]
    fn change_to_edm_cmd() {
        let resp = &[0xAAu8, 0x00, 0x02, 0x00, 0x71, 0x55];
        let correct =
            Vec::<u8, <ChangeMode as atat::AtatCmd>::CommandLen>::from_slice(b"ATO2\r\n").unwrap();
        assert_eq!(SwitchToEdmCommand.as_bytes(), correct);
        assert_eq!(SwitchToEdmCommand.parse(resp).unwrap(), NoResponse);
    }
}

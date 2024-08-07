use crate::command::edm::{
    calc_payload_len,
    types::{PayloadType, AT_COMMAND_POSITION, EDM_OVERHEAD, ENDBYTE, STARTBYTE},
};
use atat::{helpers::LossyStr, DigestResult, Digester, InternalError};

use super::edm::types::{AUTOCONNECTMESSAGE, STARTUPMESSAGE};

/// Digester for EDM context
#[derive(Debug, Default)]
pub struct EdmDigester;

impl EdmDigester {
    pub fn new() -> Self {
        Self
    }
}

impl Digester for EdmDigester {
    fn digest<'a>(&mut self, buf: &'a [u8]) -> (DigestResult<'a>, usize) {
        // TODO: Handle module restart, tests and set default startupmessage in client, and optimize this!

        if buf.is_empty() {
            return (DigestResult::None, 0);
        }

        trace!("Digest {:?}", LossyStr(buf));
        if buf.len() >= STARTUPMESSAGE.len() && buf[..2] == *b"\r\n" {
            if let Some(i) = buf[2..].windows(2).position(|x| x == *b"\r\n") {
                // Two for starting position, one for index -> len and one for the window size.
                let len = i + 4;
                trace!("Digest common at {:?}; i: {:?}", LossyStr(&buf[..len]), i);
                if buf[..len] == *STARTUPMESSAGE {
                    return (
                        DigestResult::Urc(&buf[..STARTUPMESSAGE.len()]),
                        STARTUPMESSAGE.len(),
                    );
                } else if len == AUTOCONNECTMESSAGE.len() || len == AUTOCONNECTMESSAGE.len() + 1 {
                    return (DigestResult::Urc(&buf[..len]), len);
                } else {
                    return (DigestResult::None, len);
                }
            }
        } else if buf.len() > STARTUPMESSAGE.len()
            && buf[buf.len() - STARTUPMESSAGE.len()..] == *STARTUPMESSAGE
        {
            return (
                DigestResult::Urc(&buf[buf.len() - STARTUPMESSAGE.len()..]),
                buf.len(),
            );
        }

        let start_pos = match buf.windows(1).position(|byte| byte[0] == STARTBYTE) {
            Some(pos) => pos,
            None => return (DigestResult::None, 0), // handle leading error data. // TODO: handle error input without message start.
        };

        // Trim leading invalid data.
        if start_pos != 0 {
            return (DigestResult::None, start_pos);
        }

        // Verify payload length and end byte position
        if buf.len() < EDM_OVERHEAD {
            return (DigestResult::None, 0);
        }
        let payload_len = calc_payload_len(buf);

        let edm_len = payload_len + EDM_OVERHEAD;
        if buf.len() < edm_len || buf[edm_len - 1] != ENDBYTE {
            return (DigestResult::None, 0);
        }

        // Debug statement for trace properly
        if !buf.is_empty() {
            trace!("Digest {:?}", LossyStr(buf));
        }

        // Filter message by payload
        match PayloadType::from(buf[4]) {
            PayloadType::ATConfirmation => {
                let resp = &buf[..edm_len];
                let return_val = if resp.windows(b"ERROR".len()).nth(AT_COMMAND_POSITION)
                    == Some(b"ERROR")
                    || resp.windows(b"ERROR".len()).nth(AT_COMMAND_POSITION + 2) == Some(b"ERROR")
                {
                    DigestResult::Response(Err(InternalError::InvalidResponse))
                } else {
                    DigestResult::Response(Ok(resp))
                };
                (return_val, edm_len)
            }
            PayloadType::StartEvent => (DigestResult::Response(Ok(&buf[..edm_len])), edm_len),
            PayloadType::ATEvent
            | PayloadType::ConnectEvent
            | PayloadType::DataEvent
            | PayloadType::DisconnectEvent => {
                // Received EDM event
                (DigestResult::Urc(&buf[..edm_len]), edm_len)
            }
            _ => {
                // Wrong/Unsupported packet, thrown away.
                (DigestResult::None, edm_len)
            }
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use atat::Config;
//     use atat::{AtatIngress, Buffers, Response, blocking::AtatClient};

//     const TEST_RX_BUF_LEN: usize = 256;
//     const TEST_RES_CAPACITY: usize = 3 * TEST_RX_BUF_LEN;
//     const TEST_URC_CAPACITY: usize = 3 * TEST_RX_BUF_LEN;

//     struct MockWriter;

//     /// Removed functionality used to change OK responses to empty responses.
//     #[test]
//     fn ok_response<'a>() {
//         let buf = Buffers::<TEST_RX_BUF_LEN, TEST_RES_CAPACITY, TEST_URC_CAPACITY>::new();
//         (at_pars, client) = buf.split_blocking(MockWriter, EdmDigester::default(), Config::new());

//         // Payload: "OK\r\n"
//         let data = &[0xAA, 0x00, 0x06, 0x00, 0x45, 0x4f, 0x4b, 0x0D, 0x0a, 0x55];
//         let empty_ok_response = &[0xAA, 0x00, 0x06, 0x00, 0x45, 0x4f, 0x4b, 0x0D, 0x0a, 0x55];

//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len).unwrap();

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);
//         let frame = Frame::decode(grant.as_ref());
//         let res = match Response::from(frame) {
//             Response::Result(r) => r,
//             Response::Prompt(_) => Ok(&[][..]),
//         };

//         assert_eq!(res, Ok(&empty_ok_response[..]));
//         assert_eq!(urc_c.read(), None);
//     }
// }

//     #[test]
//     fn error_response() {
//         let mut at_pars: Ingress<
//             'static,
//             EdmDigester,
//             TEST_RX_BUF_LEN,
//             TEST_RES_CAPACITY,
//             TEST_URC_CAPACITY,
//         >;
//         let mut res_c: FrameConsumer<'static, TEST_RES_CAPACITY>;
//         let mut urc_c: FrameConsumer<'static, TEST_URC_CAPACITY>;
//         (at_pars, res_c, urc_c) =
//             Buffers::<TEST_RX_BUF_LEN, TEST_RES_CAPACITY, TEST_URC_CAPACITY>::new()
//                 .to_ingress(EdmDigester::default());

//         // Payload: "ERROR\r\n"
//         let data = &[
//             0xAA, 0x00, 0x09, 0x00, 0x45, 0x45, 0x52, 0x52, 0x4f, 0x52, 0x0D, 0x0a, 0x55,
//         ];

//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);
//         let frame = Frame::decode(grant.as_ref());
//         let res = match Response::from(frame) {
//             Response::Result(r) => r,
//             Response::Prompt(_) => Ok(&[][..]),
//         };
//         assert_eq!(res, Err(InternalError::InvalidResponse));
//         assert_eq!(urc_c.read(), None);
//     }

//     #[test]
//     fn regular_response_with_trailing_ok() {
//         let mut at_pars: Ingress<
//             'static,
//             EdmDigester,
//             TEST_RX_BUF_LEN,
//             TEST_RES_CAPACITY,
//             TEST_URC_CAPACITY,
//         >;
//         let mut res_c: FrameConsumer<'static, TEST_RES_CAPACITY>;
//         let mut urc_c: FrameConsumer<'static, TEST_URC_CAPACITY>;
//         (at_pars, res_c, urc_c) =
//             Buffers::<TEST_RX_BUF_LEN, TEST_RES_CAPACITY, TEST_URC_CAPACITY>::new()
//                 .to_ingress(EdmDigester::default());

//         // Payload: AT\r\n
//         let response = &[0xAA, 0x00, 0x06, 0x00, 0x45, 0x41, 0x54, 0x0D, 0x0a, 0x55];
//         // Data = response + trailing OK message
//         let data = &[
//             0xAA, 0x00, 0x06, 0x00, 0x45, 0x41, 0x54, 0x0D, 0x0a, 0x55, 0xAA, 0x00, 0x06, 0x00,
//             0x45, 0x4f, 0x4b, 0x0D, 0x0a, 0x55,
//         ];

//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);
//         let frame = Frame::decode(grant.as_ref());
//         let res = match Response::from(frame) {
//             Response::Result(r) => r,
//             Response::Prompt(_) => Ok(&[][..]),
//         };
//         assert_eq!(res, Ok(&response[..]));
//         assert_eq!(urc_c.read(), None);
//     }

//     /// Regular response with trailing regular response..
//     #[test]
//     fn at_urc() {
//         let mut at_pars: Ingress<
//             'static,
//             EdmDigester,
//             TEST_RX_BUF_LEN,
//             TEST_RES_CAPACITY,
//             TEST_URC_CAPACITY,
//         >;
//         let mut res_c: FrameConsumer<'static, TEST_RES_CAPACITY>;
//         let mut urc_c: FrameConsumer<'static, TEST_URC_CAPACITY>;
//         (at_pars, res_c, urc_c) =
//             Buffers::<TEST_RX_BUF_LEN, TEST_RES_CAPACITY, TEST_URC_CAPACITY>::new()
//                 .to_ingress(EdmDigester::default());

//         let type_byte = PayloadType::ATEvent as u8;
//         // Payload: "OK\r\n"
//         let data = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let result = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);
//         assert_eq!(grant.as_ref(), result);
//         assert_eq!(res_c.read(), None);
//     }

//     #[test]
//     fn data_event() {
//         let mut at_pars: Ingress<
//             'static,
//             EdmDigester,
//             TEST_RX_BUF_LEN,
//             TEST_RES_CAPACITY,
//             TEST_URC_CAPACITY,
//         >;
//         let mut res_c: FrameConsumer<'static, TEST_RES_CAPACITY>;
//         let mut urc_c: FrameConsumer<'static, TEST_URC_CAPACITY>;
//         (at_pars, res_c, urc_c) =
//             Buffers::<TEST_RX_BUF_LEN, TEST_RES_CAPACITY, TEST_URC_CAPACITY>::new()
//                 .to_ingress(EdmDigester::default());

//         let type_byte = PayloadType::DataEvent as u8;
//         // Payload: "OK\r\n"
//         let data = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let result = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);

//         assert_eq!(grant.as_ref(), result);
//         assert_eq!(res_c.read(), None);
//     }

//     #[test]
//     fn connect_disconnect_events() {
//         let mut at_pars: Ingress<
//             'static,
//             EdmDigester,
//             TEST_RX_BUF_LEN,
//             TEST_RES_CAPACITY,
//             TEST_URC_CAPACITY,
//         >;
//         let mut res_c: FrameConsumer<'static, TEST_RES_CAPACITY>;
//         let mut urc_c: FrameConsumer<'static, TEST_URC_CAPACITY>;
//         (at_pars, res_c, urc_c) =
//             Buffers::<TEST_RX_BUF_LEN, TEST_RES_CAPACITY, TEST_URC_CAPACITY>::new()
//                 .to_ingress(EdmDigester::default());

//         let type_byte = PayloadType::ConnectEvent as u8;
//         // Payload: "OK\r\n"
//         let data = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let result = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);
//         assert_eq!(grant.as_ref(), result);
//         assert_eq!(res_c.read(), None);
//         drop(grant);

//         let type_byte = PayloadType::DisconnectEvent as u8;
//         // Payload: "OK\r\n"
//         let data = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let result = &[
//             0xAA, 0x00, 0x0E, 0x00, type_byte, 0x0D, 0x0A, 0x2B, 0x55, 0x55, 0x44, 0x50, 0x44,
//             0x3A, 0x33, 0x0D, 0x0A, 0x55,
//         ];
//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);

//         let mut grant = urc_c.read().unwrap();
//         grant.auto_release(true);
//         assert_eq!(grant.as_ref(), result);
//         assert_eq!(res_c.read(), None);
//     }

//     #[test]
//     fn wrong_type_packet() {
//         let mut at_pars: Ingress<
//             'static,
//             EdmDigester,
//             TEST_RX_BUF_LEN,
//             TEST_RES_CAPACITY,
//             TEST_URC_CAPACITY,
//         >;
//         let mut res_c: FrameConsumer<'static, TEST_RES_CAPACITY>;
//         let mut urc_c: FrameConsumer<'static, TEST_URC_CAPACITY>;
//         (at_pars, res_c, urc_c) =
//             Buffers::<TEST_RX_BUF_LEN, TEST_RES_CAPACITY, TEST_URC_CAPACITY>::new()
//                 .to_ingress(EdmDigester::default());

//         let type_byte = PayloadType::Unknown as u8;
//         // Payload: "OK\r\n"
//         let data = &[
//             0xAA, 0x00, 0x06, 0x00, type_byte, 0x4f, 0x4b, 0x0D, 0x0a, 0x55,
//         ];
//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);

//         assert_eq!(urc_c.read(), None);
//         assert_eq!(res_c.read(), None);

//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);
//         assert_eq!(urc_c.read(), None);
//         assert_eq!(res_c.read(), None);

//         // Recovered enough to receive normal data?
//         // Payload: "OK\r\n"
//         let data = &[0xAA, 0x00, 0x06, 0x00, 0x45, 0x4f, 0x4b, 0x0D, 0x0a, 0x55];
//         let empty_ok_response = heapless::Vec::<u8, TEST_RX_BUF_LEN>::from_slice(&[
//             0xAA, 0x00, 0x06, 0x00, 0x45, 0x4f, 0x4b, 0x0D, 0x0a, 0x55,
//         ])
//         .unwrap();

//         let ingress_buf = at_pars.write_buf();
//         let len = usize::min(data.len(), ingress_buf.len());
//         ingress_buf[..len].copy_from_slice(&data[..len]);
//         at_pars.try_advance(len);

//         let mut grant = res_c.read().unwrap();
//         grant.auto_release(true);

//         let frame = Frame::decode(grant.as_ref());
//         let res = match Response::from(frame) {
//             Response::Result(r) => r,
//             Response::Prompt(_) => Ok(&[][..]),
//         };
//         assert_eq!(res, Ok(&empty_ok_response[..]));
//         assert_eq!(urc_c.read(), None);
//     }
// }

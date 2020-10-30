use atat::atat_log;
use atat::error::Error;
use atat::ingress_manager::{
    get_line, IngressManager, SliceExt, State, UrcMatcher, UrcMatcherResult, ByteVec,
};
use atat::queues::{ComItem, ResItem, UrcItem};
use atat::heapless::{ArrayLength, Vec};
use crate::command::edm::{EDM_OVERHEAD, EDM_FULL_SIZE_FILTER, PayloadType};

/// Custom function to process the receive buffer, checking for AT responses, URC's or errors
///
/// This function should be called regularly for the ingress manager to work
pub(crate) fn custom_digest<BufLen, U, ComCapacity, ResCapacity, UrcCapacity>(
    ingress: &mut IngressManager<BufLen, U, ComCapacity, ResCapacity, UrcCapacity>,
) where
    U: UrcMatcher<BufLen>,
    BufLen: ArrayLength<u8>,
    ComCapacity: ArrayLength<ComItem>,
    ResCapacity: ArrayLength<ResItem<BufLen>>,
    UrcCapacity: ArrayLength<UrcItem<BufLen>>,
{
    // Handle commands
    ingress.handle_com();

    let end_byte = ingress.get_line_term_char();
    let start_byte = ingress.get_format_char();
    // Echo is currently not suported in EDM
    if ingress.get_echo_enabled() {
        unimplemented!("Enabeling echo is currently unsupported for EDM");
    }

    let start_pos = match ingress.buf.windows(1).position(|byte| byte[0] == start_byte){
        Some(pos) => pos,
        None => return,
    };

    // Trim leading invalid data.
    if start_pos != 0 {
        ingress.buf = Vec::from_slice(&ingress.buf[start_pos.. ingress.buf.len()]).unwrap();
    }

    // Verify payload length and end byte position
    if ingress.buf.len() < EDM_OVERHEAD{
        return;
    }
    let payload_len = (((ingress.buf[1] as u16) << 8 | ingress.buf[2] as u16) & EDM_FULL_SIZE_FILTER) as usize;

    let edm_len = payload_len + EDM_OVERHEAD;
    if ingress.buf.len() < edm_len {
        return;
    } else if ingress.buf[edm_len -1] != end_byte{
        return;
    }

    match PayloadType::from(ingress.buf[4]) {
        PayloadType::ATConfirmation => {
            let (resp, mut remaining) = ingress.buf.split_at(edm_len);
            let mut return_val: Option<Result<ByteVec<BufLen>, Error>> = None;
            if ingress.get_state() == State::ReceivingResponse {    
                if let Some(_) = resp.windows(b"ERROR".len()).position(|window| window == b"ERROR" ) {
                    //Recieved Error response
                    return_val = Some(Err(Error::InvalidResponse));
                } else if let Some(_) = resp.windows(b"OK".len()).position(|window| window == b"OK" ) {
                    //Recieved OK response
                    return_val = Some(Ok(ByteVec::<BufLen>::from_slice(&[
                        0xAAu8,
                        0x00,
                        0x02,
                        0x00,
                        PayloadType::ATConfirmation as u8,
                        0x55,
                        ]).unwrap()));
                } else {
                    //Normal response check if OK recived at end? else return to wait for OK to be received at end.
                    let start_pos_remaining = match remaining.windows(1).position(|byte| byte == &[start_byte]){
                        Some(pos) => pos,
                        None => return,
                    };
            
                    if start_pos_remaining != 0 {
                        remaining = &remaining[start_pos_remaining .. remaining.len()];
                    }
            
                    if remaining.len() < EDM_OVERHEAD{
                        return;
                    }
                    let payload_len_remaining = (((remaining[1] as u16) << 8 | remaining[2] as u16) & EDM_FULL_SIZE_FILTER) as usize;
                    let edm_len_remaining = payload_len_remaining + EDM_OVERHEAD;
                    if remaining.len() < edm_len_remaining {
                        return;
                    } else if remaining[edm_len_remaining -1] != end_byte{
                        return;
                    }
                    if PayloadType::from(remaining[4]) == PayloadType::ATConfirmation 
                        && remaining.windows(b"OK".len()).position(|window| window == b"OK" ) != None {
                        // Found trailing OK response remove from remaining
                        remaining = &remaining[edm_len_remaining .. remaining.len()];

                    } // else next response not OK?... TODO: Handle this case
                    return_val = Some(Ok(ByteVec::<BufLen>::from_slice(resp).unwrap()))
                }
            }
            ingress.buf = Vec::from_slice(remaining).unwrap();
            if let Some(resp) = return_val {
                ingress.notify_response(resp)
            }
        },
        PayloadType::ATEvent=> {
            // Recived URC
            let (resp, remaining) = ingress.buf.split_at(edm_len);
            let resp = ByteVec::<BufLen>::from_slice(resp).unwrap();
            ingress.buf = Vec::from_slice(remaining).unwrap();
            ingress.notify_urc(resp);
        }
        _ => {
            // Wrong/Unsupported packet, thrown away.
            let (resp, remaining) = ingress.buf.split_at(edm_len);
            ingress.buf = Vec::from_slice(remaining).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::command::edm::{STARTBYTE, ENDBYTE};
    use atat::ingress_manager::{NoopUrcMatcher, ByteVec};
    use atat::queues::{ComQueue, ResQueue, UrcQueue};
    use atat::{Mode, Config};
    use atat::heapless::{consts, spsc::Queue};

    type TestRxBufLen = consts::U256;
    type TestComCapacity = consts::U3;
    type TestResCapacity = consts::U5;
    type TestUrcCapacity = consts::U10;

    macro_rules! setup {
        ($config:expr, $urch:expr) => {{
            static mut RES_Q: ResQueue<TestRxBufLen, TestResCapacity> =
                Queue(heapless::i::Queue::u8());
            let (res_p, res_c) = unsafe { RES_Q.split() };
            static mut URC_Q: UrcQueue<TestRxBufLen, TestUrcCapacity> =
                Queue(heapless::i::Queue::u8());
            let (urc_p, urc_c) = unsafe { URC_Q.split() };
            static mut COM_Q: ComQueue<TestComCapacity> = Queue(heapless::i::Queue::u8());
            let (_com_p, com_c) = unsafe { COM_Q.split() };
            (
                IngressManager::with_customs(res_p, urc_p, com_c, $config, $urch, custom_digest),
                res_c,
                urc_c,
            )
        }};
        ($config:expr) => {{
            let val: (
                IngressManager<
                    TestRxBufLen,
                    NoopUrcMatcher,
                    TestComCapacity,
                    TestResCapacity,
                    TestUrcCapacity,
                >,
                _,
                _,
            ) = setup!($config, None);
            val
        }};
    }
    
    #[test]
    fn ok_response() {
        let conf = Config::new(Mode::Timeout).with_at_echo(false).with_line_term(ENDBYTE).with_format_char(STARTBYTE);
        let (mut at_pars, mut res_c, mut urc_c) = setup!(conf);

        assert_eq!(at_pars.get_state(), State::Idle);

        at_pars.set_state(State::ReceivingResponse);
                                                            //  O   K   \r   \n
        let data = &[0xAAu8,0x00,0x06,0x00,0x45,0x4f,0x4b,0x0D,0x0a,0x55];
        let empty_ok_response = 
            Vec::<u8, TestRxBufLen>::from_slice(&[ 0xAAu8, 0x00, 0x02, 0x00, PayloadType::ATConfirmation as u8, 0x55]).unwrap();

        at_pars.write(data);
        assert_eq!(at_pars.buf, Vec::<u8, TestRxBufLen>::from_slice(data).unwrap());

        at_pars.digest();
        assert_eq!(at_pars.buf, Vec::<_, TestRxBufLen>::new());
        assert_eq!(res_c.dequeue(), Some(Ok(empty_ok_response)));
        assert_eq!(urc_c.dequeue(), None);
    }

    #[test]
    fn error_response() {
        let conf = Config::new(Mode::Timeout).with_at_echo(false).with_line_term(ENDBYTE).with_format_char(STARTBYTE);
        let (mut at_pars, mut res_c, mut urc_c) = setup!(conf);

        assert_eq!(at_pars.get_state(), State::Idle);

        at_pars.set_state(State::ReceivingResponse);
                                                        //  E    R    R    O    R   \r   \n
        let data = &[0xAAu8,0x00,0x09,0x00,0x45,0x45,0x52,0x52,0x4f,0x52,0x0D,0x0a,0x55];

        at_pars.write(data);
        assert_eq!(at_pars.buf, Vec::<u8, TestRxBufLen>::from_slice(data).unwrap());

        at_pars.digest();
        assert_eq!(at_pars.buf, Vec::<_, TestRxBufLen>::new());
        assert_eq!(res_c.dequeue(), Some(Err(Error::InvalidResponse)));
        assert_eq!(urc_c.dequeue(), None);
    }

    #[test]
    fn regular_response_with_trailing_ok() {
        let conf = Config::new(Mode::Timeout).with_at_echo(false).with_line_term(ENDBYTE).with_format_char(STARTBYTE);
        let (mut at_pars, mut res_c, mut urc_c) = setup!(conf);

        assert_eq!(at_pars.get_state(), State::Idle);

        at_pars.set_state(State::ReceivingResponse);
                                                                //  A   T   \r   \n
        let response = &[0xAAu8,0x00,0x06,0x00,0x45,0x41,0x54,0x0D,0x0a,0x55];
        // Data = response + trailing OK message
        let data = &[0xAAu8,0x00,0x06,0x00,0x45,0x41,0x54,0x0D,0x0a,0x55,0xAA,0x00,0x06,0x00,0x45,0x4f,0x4b,0x0D,0x0a,0x55];

        at_pars.write(data);
        assert_eq!(at_pars.buf, Vec::<u8, TestRxBufLen>::from_slice(data).unwrap());

        at_pars.digest();
        assert_eq!(at_pars.buf, Vec::<_, TestRxBufLen>::new());
        assert_eq!(res_c.dequeue(), Some(Ok(Vec::<u8, TestRxBufLen>::from_slice(response).unwrap())));
        assert_eq!(urc_c.dequeue(), None);
    }

    #[test]
    fn urc() {
        let conf = Config::new(Mode::Timeout).with_at_echo(false).with_line_term(ENDBYTE).with_format_char(STARTBYTE);
        let (mut at_pars, mut res_c, mut urc_c) = setup!(conf);

        assert_eq!(at_pars.get_state(), State::Idle);

        let type_byte = PayloadType::ATEvent as u8;
                                                            //  O   K   \r   \n
        let data = &[0xAAu8, 0x00, 0x06, 0x00, type_byte, 0x4f, 0x4b, 0x0D, 0x0a, 0x55];
        at_pars.write(data);
        assert_eq!(at_pars.buf, Vec::<u8, TestRxBufLen>::from_slice(data).unwrap());
        at_pars.digest();
        assert_eq!(at_pars.buf, Vec::<_, TestRxBufLen>::new());
        assert_eq!(urc_c.dequeue(), Some(Vec::<u8, TestRxBufLen>::from_slice(data).unwrap()));
        assert_eq!(res_c.dequeue(), None);
    }

    #[test]
    fn wrong_type_packet() {
        let conf = Config::new(Mode::Timeout).with_at_echo(false).with_line_term(ENDBYTE).with_format_char(STARTBYTE);
        let (mut at_pars, mut res_c, mut urc_c) = setup!(conf);

        assert_eq!(at_pars.get_state(), State::Idle);

        let type_byte = PayloadType::Unknown as u8;
                                                            //  O   K   \r   \n
        let data = &[0xAAu8, 0x00, 0x06, 0x00, type_byte, 0x4f, 0x4b, 0x0D, 0x0a, 0x55];
        at_pars.write(data);
        assert_eq!(at_pars.buf, Vec::<u8, TestRxBufLen>::from_slice(data).unwrap());
        at_pars.digest();
        assert_eq!(at_pars.buf, Vec::<_, TestRxBufLen>::new());
        assert_eq!(urc_c.dequeue(), None);
        assert_eq!(res_c.dequeue(), None);
    }

}
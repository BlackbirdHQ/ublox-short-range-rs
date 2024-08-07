//! Responses for GPIO Commands
use super::types::*;
use atat::atat_derive::AtatResp;

/// 14.2 GPIO Read +UGPIOR
#[derive(Clone, PartialEq, AtatResp)]
pub struct ReadGPIOResponse {
    #[at_arg(position = 0)]
    pub id: GPIOId,
    #[at_arg(position = 1)]
    pub value: GPIOValue,
}

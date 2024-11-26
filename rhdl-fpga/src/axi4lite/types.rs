// The data types that pass through the write address channel.

// The valid and ready signals are handled by the channel.

use std::fmt::Write;

use rhdl::prelude::*;

use super::channel::{ChannelRToS, ChannelSToR};

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct WriteAddress<ID: Digital, const ADDR: usize> {
    /// The ID of the transaction.  Can be any digital type.
    pub id: ID,
    /// Address of the transaction (this is a byte address per the specification)
    pub addr: Bits<ADDR>,
}

impl<ID: Digital, const ADDR: usize> Default for WriteAddress<ID, ADDR> {
    fn default() -> Self {
        Self {
            id: ID::init(),
            addr: bits(0),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct BurstData<DATA: Digital> {
    // Data
    pub data: DATA,
    // Last data in the burst
    pub last: bool,
}

impl<DATA: Digital> Default for BurstData<DATA> {
    fn default() -> Self {
        Self {
            data: DATA::init(),
            last: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
pub enum ResponseKind {
    #[default]
    OKAY = 0,
    EXOKAY = 1,
    SLVERR = 2,
    DECERR = 3,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct WriteResponse<ID: Digital> {
    /// The ID of the transaction.  Can be any digital type.
    pub id: ID,
    /// The response to the transaction
    pub resp: ResponseKind,
}

impl<ID: Digital> Default for WriteResponse<ID> {
    fn default() -> Self {
        Self {
            id: ID::init(),
            resp: ResponseKind::OKAY,
        }
    }
}

// We need inputs for the bus of each channel
#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct AddrWrite<ID: Digital, DATA: Digital, const ADDR: usize> {
    pub addr: ChannelSToR<WriteAddress<ID, ADDR>>,
    pub data: ChannelSToR<DATA>,
    pub resp: ChannelRToS,
}

// We need outputs for each of the channels
#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct AddrRead<ID: Digital, const ADDR: usize> {
    pub addr: ChannelRToS,
    pub data: ChannelRToS,
    pub resp: ChannelSToR<WriteResponse<ID>>,
}

use rhdl::prelude::*;

pub mod sink;
pub mod source;
pub mod testing;

#[derive(Debug, Digital, Default)]
pub struct StreamMOSI<T: Digital + Default> {
    /// The data to be sent
    pub tdata: T,
    /// The data valid flag
    pub tvalid: bool,
}

#[derive(Debug, Digital, Default)]
pub struct StreamMISO {
    /// The ready flag from the consumer
    pub tready: bool,
}

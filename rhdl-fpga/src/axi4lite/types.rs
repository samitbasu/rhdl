// The data types that pass through the write address channel.
// The valid and ready signals are handled by the channel.
use rhdl::prelude::*;

use super::channel::{DataValid, Ready};

pub type ResponseKind = Bits<2>;

// The response kinds
pub mod response_codes {
    use rhdl::prelude::*;

    pub const OKAY: Bits<2> = bits(0);
    pub const EXOKAY: Bits<2> = bits(1);
    pub const SLVERR: Bits<2> = bits(2);
    pub const DECERR: Bits<2> = bits(3);
}

#[derive(Debug, Digital)]
pub struct ReadResponse<const DATA: usize = 32> {
    /// The response to the transaction
    pub resp: ResponseKind,
    /// The data to return
    pub data: Bits<DATA>,
}

impl<const DATA: usize> Default for ReadResponse<DATA> {
    fn default() -> Self {
        Self {
            resp: response_codes::OKAY,
            data: Bits::dont_care(),
        }
    }
}

/*

  input  wire [AXI_ADDR_WIDTH-1:0]   s_axi_araddr,  // AXI4-Lite slave: Read address
  input  wire                        s_axi_arvalid, // AXI4-Lite slave: Read address valid
  input  wire                        s_axi_rready   // AXI4-Lite slave: Read data ready

  output wire                        s_axi_arready, // AXI4-Lite slave: Read address ready
  output wire [AXI_DATA_WIDTH-1:0]   s_axi_rdata,   // AXI4-Lite slave: Read data
  output wire [1:0]                  s_axi_rresp,   // AXI4-Lite slave: Read data response
  output wire                        s_axi_rvalid,  // AXI4-Lite slave: Read data valid

*/

#[derive(Debug, Digital)]
pub struct ReadMOSI<const ADDR: usize = 32> {
    /// Read Address
    pub araddr: Bits<ADDR>,
    /// Read Address valid
    pub arvalid: bool,
    /// Read Data ready
    pub rready: bool,
}

#[derive(Debug, Digital)]
pub struct ReadMISO<const DATA: usize = 32> {
    /// Read Address ready
    pub arready: bool,
    /// Read Data
    pub rdata: Bits<DATA>,
    /// Read Data response
    pub rresp: Bits<2>,
    /// Read Data valid
    pub rvalid: bool,
}

#[derive(Debug, Digital)]
pub struct WriteMOSI<const DATA: usize = 32, const ADDR: usize = 32> {
    /// Write Address
    pub awaddr: Bits<ADDR>,
    /// Write Address valid
    pub awvalid: bool,
    /// Write Data
    pub wdata: Bits<DATA>,
    /// Write Data valid
    pub wvalid: bool,
    /// Write Response ready
    pub bready: bool,
}

#[derive(Debug, Digital)]
pub struct WriteMISO {
    /// Write Address ready
    pub awready: bool,
    /// Write Data ready
    pub wready: bool,
    /// Write Response
    pub bresp: Bits<2>,
    /// Write Response valid
    pub bvalid: bool,
}

#[derive(Debug, Digital)]
pub struct MOSI<const DATA: usize = 32, const ADDR: usize = 32> {
    pub read: ReadMOSI<ADDR>,
    pub write: WriteMOSI<DATA, ADDR>,
}

#[derive(Debug, Digital)]
pub struct MISO<const DATA: usize = 32> {
    pub read: ReadMISO<DATA>,
    pub write: WriteMISO,
}

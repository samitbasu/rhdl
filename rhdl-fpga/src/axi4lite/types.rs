// The data types that pass through the write address channel.
// The valid and ready signals are handled by the channel.
use rhdl::prelude::*;

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
pub struct ReadResponse<const DATA: usize> {
    /// The response to the transaction
    pub resp: ResponseKind,
    /// The data to return
    pub data: Bits<DATA>,
}

impl<const DATA: usize> Default for ReadResponse<DATA> {
    fn default() -> Self {
        Self {
            resp: response_codes::OKAY,
            data: Bits::default(),
        }
    }
}

// An AXI4-Error Enum meant to capture the two cases of
// SLVERR and DECERR
#[derive(Debug, Digital, Default)]
pub enum AXI4Error {
    #[default]
    SLVERR,
    DECERR,
}

#[kernel]
pub fn read_response_to_result<const DATA: usize>(
    resp: ReadResponse<DATA>,
) -> Result<Bits<DATA>, AXI4Error> {
    match resp.resp {
        response_codes::OKAY => Ok(resp.data),
        response_codes::EXOKAY => Ok(resp.data),
        response_codes::DECERR => Err(AXI4Error::DECERR),
        _ => Err(AXI4Error::SLVERR),
    }
}

#[kernel]
pub fn result_to_read_response<const DATA: usize>(
    resp: Result<Bits<DATA>, AXI4Error>,
) -> ReadResponse<DATA> {
    match resp {
        Ok(data) => ReadResponse::<DATA> {
            resp: response_codes::OKAY,
            data,
        },
        Err(e) => match e {
            AXI4Error::SLVERR => ReadResponse::<DATA> {
                resp: response_codes::SLVERR,
                data: bits(0),
            },
            AXI4Error::DECERR => ReadResponse::<DATA> {
                resp: response_codes::DECERR,
                data: bits(0),
            },
        },
    }
}

#[kernel]
pub fn result_to_write_response(resp: Result<(), AXI4Error>) -> ResponseKind {
    match resp {
        Ok(_) => response_codes::OKAY,
        Err(e) => match e {
            AXI4Error::SLVERR => response_codes::SLVERR,
            AXI4Error::DECERR => response_codes::DECERR,
        },
    }
}

#[kernel]
pub fn write_response_to_result(resp: ResponseKind) -> Result<(), AXI4Error> {
    match resp {
        response_codes::OKAY => Ok(()),
        response_codes::EXOKAY => Ok(()),
        response_codes::DECERR => Err(AXI4Error::DECERR),
        _ => Err(AXI4Error::SLVERR),
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
pub struct ReadMOSI<const ADDR: usize> {
    /// Read Address
    pub araddr: Bits<ADDR>,
    /// Read Address valid
    pub arvalid: bool,
    /// Read Data ready
    pub rready: bool,
}

#[derive(Debug, Digital, Default)]
pub struct ReadMISO<const DATA: usize> {
    /// Read Address ready
    pub arready: bool,
    /// Read Data
    pub rdata: Bits<DATA>,
    /// Read Data response
    pub rresp: Bits<2>,
    /// Read Data valid
    pub rvalid: bool,
}

#[derive(Debug, Digital, Default)]
pub struct WriteMOSI<const DATA: usize, const ADDR: usize> {
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
pub struct MOSI<const DATA: usize, const ADDR: usize> {
    pub read: ReadMOSI<ADDR>,
    pub write: WriteMOSI<DATA, ADDR>,
}

#[derive(Debug, Digital)]
pub struct MISO<const DATA: usize> {
    pub read: ReadMISO<DATA>,
    pub write: WriteMISO,
}

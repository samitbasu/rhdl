// The data types that pass through the write address channel.
// The valid and ready signals are handled by the channel.
use rhdl::prelude::*;

pub type ResponseKind = Bits<W2>;

// The response kinds
pub mod response_codes {
    use rhdl::prelude::*;

    pub const OKAY: b2 = bits(0);
    pub const EXOKAY: b2 = bits(1);
    pub const SLVERR: b2 = bits(2);
    pub const DECERR: b2 = bits(3);
}

pub type AxilData = Bits<W32>;
pub type AxilAddr = Bits<W32>;
pub type AxilStrobe = Bits<W4>;

#[derive(PartialEq, Debug, Digital, Default)]
pub struct StrobedData {
    /// The data to write
    pub data: AxilData,
    /// The strobe to use
    pub strobe: AxilStrobe,
}

#[derive(PartialEq, Debug, Digital)]
pub struct ReadResponse {
    /// The response to the transaction
    pub resp: ResponseKind,
    /// The data to return
    pub data: AxilData,
}

impl Default for ReadResponse {
    fn default() -> Self {
        Self {
            resp: response_codes::OKAY,
            data: Bits::default(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Default)]
pub struct WriteCommand {
    /// The address to write to
    pub addr: AxilAddr,
    /// The data to write along with the strobe
    pub strobed_data: StrobedData,
}

// convert a strobe into a mask
#[kernel]
pub fn strobe_to_mask(strobe: Bits<W4>) -> Bits<W32> {
    let mut mask = bits(0);
    if strobe & 1 != 0 {
        mask |= bits(0xff);
    }
    if strobe & 2 != 0 {
        mask |= bits(0xff00);
    }
    if strobe & 4 != 0 {
        mask |= bits(0xff0000);
    }
    if strobe & 8 != 0 {
        mask |= bits(0xff000000);
    }
    mask
}

// An AXI4-Error Enum meant to capture the two cases of
// SLVERR and DECERR
#[derive(PartialEq, Debug, Digital, Default)]
pub enum AXI4Error {
    #[default]
    SLVERR,
    DECERR,
}

#[kernel]
pub fn read_response_to_result(resp: ReadResponse) -> Result<AxilData, AXI4Error> {
    match resp.resp {
        response_codes::OKAY => Ok(resp.data),
        response_codes::EXOKAY => Ok(resp.data),
        response_codes::DECERR => Err(AXI4Error::DECERR),
        _ => Err(AXI4Error::SLVERR),
    }
}

#[kernel]
pub fn result_to_read_response(resp: Result<AxilData, AXI4Error>) -> ReadResponse {
    match resp {
        Ok(data) => ReadResponse {
            resp: response_codes::OKAY,
            data,
        },
        Err(e) => match e {
            AXI4Error::SLVERR => ReadResponse {
                resp: response_codes::SLVERR,
                data: bits(0),
            },
            AXI4Error::DECERR => ReadResponse {
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

#[derive(PartialEq, Debug, Digital)]
pub struct ReadMOSI {
    /// Read Address
    pub araddr: AxilAddr,
    /// Read Address valid
    pub arvalid: bool,
    /// Read Data ready
    pub rready: bool,
}

#[derive(PartialEq, Debug, Digital, Default)]
pub struct ReadMISO {
    /// Read Address ready
    pub arready: bool,
    /// Read Data
    pub rdata: AxilData,
    /// Read Data response
    pub rresp: Bits<W2>,
    /// Read Data valid
    pub rvalid: bool,
}

#[derive(PartialEq, Debug, Digital, Default)]
pub struct WriteMOSI {
    /// Write Address
    pub awaddr: AxilAddr,
    /// Write Address valid
    pub awvalid: bool,
    /// Write Data
    pub wdata: AxilData,
    /// Write byte strobe
    pub wstrb: AxilStrobe,
    /// Write Data valid
    pub wvalid: bool,
    /// Write Response ready
    pub bready: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct WriteMISO {
    /// Write Address ready
    pub awready: bool,
    /// Write Data ready
    pub wready: bool,
    /// Write Response
    pub bresp: Bits<W2>,
    /// Write Response valid
    pub bvalid: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct MOSI {
    pub read: ReadMOSI,
    pub write: WriteMOSI,
}

#[derive(PartialEq, Debug, Digital)]
pub struct MISO {
    pub read: ReadMISO,
    pub write: WriteMISO,
}

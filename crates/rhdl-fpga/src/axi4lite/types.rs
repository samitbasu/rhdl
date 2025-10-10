//! The AXI bus types
//!
//! This module includes basic types and type definitions required
//! by the AXI specification.  Although there are clearly more
//! ergonomic ways to represent these values in `rhdl` (the [ResponseKind],
//! for example, could easily be an `enum`), we use basic raw
//! types here since we need to export these signals to other cores
//! that are likely not implemented in `rhdl`.

use rhdl::prelude::*;

/// The [ResponseKind] called out in the AXI specification
pub type ResponseKind = Bits<U2>;

/// This module [response_codes] includes definitions
/// for the 4 responses defined in the specification
pub mod response_codes {
    use rhdl::prelude::*;

    /// The specified response for an OK condition
    pub const OKAY: b2 = bits(0);
    /// Exclusive access granted
    pub const EXOKAY: b2 = bits(1);
    /// Slave Error - i.e., the endpoint has reported an error
    pub const SLVERR: b2 = bits(2);
    /// Decode Error - i.e., the provided transaction could not be decoded
    pub const DECERR: b2 = bits(3);
}

/// AXI Data type
///
/// Although the specification allows for AXI data busses of any width,
/// we implement the 32-bit width in this module.
pub type AxilData = Bits<U32>;
/// AXI Address type
///
/// The specification also allows for various widths of AXI addresses.
/// Because 32 bits is the most common format, we choose that here.
pub type AxilAddr = Bits<U32>;
/// AXI Strobe Mask
///
/// This data type includes a bit-per-byte of the [AxilData] type.  Thus
/// it should have `N` bits where [AxilData] has `2^(8*N)` bits.
pub type AxilStrobe = Bits<U4>;

#[derive(PartialEq, Debug, Digital, Clone, Default)]
/// A data word to write, along with a strobe for the bytes
pub struct StrobedData {
    /// The data to write
    pub data: AxilData,
    /// The strobe to use
    pub strobe: AxilStrobe,
}

#[derive(PartialEq, Debug, Digital, Clone)]
/// The response to a read transaction
///
/// The spec indicates that a read transaction
/// returns data and a [ResponseKind].  
/// Presumably, if the [ResponseKind] indicates
/// an error, then we should ignore the data lines.
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

#[derive(PartialEq, Debug, Digital, Clone, Default)]
/// The write command
///
/// The address and strobed data are sent on
/// different channels, but combined in this
/// data structure.
pub struct WriteCommand {
    /// The address to write to
    pub addr: AxilAddr,
    /// The data to write along with the strobe
    pub strobed_data: StrobedData,
}

#[kernel]
/// Convert a strobe into a mask
///
/// This function simply converts the strobe into
/// a mask.
pub fn strobe_to_mask(strobe: Bits<U4>) -> Bits<U32> {
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

#[derive(PartialEq, Debug, Digital, Clone, Default)]
/// An AXI4-Error Enum meant to capture the two cases of
/// SLVERR and DECERR.  The [AXI4Error] enum is
/// easier to work with in `rhdl`.
pub enum AXI4Error {
    #[default]
    /// A slave device raised an error.
    SLVERR,
    /// The transaction decode failed.
    DECERR,
}

#[derive(PartialEq, Debug, Digital, Clone, Default)]
/// Flag to indicate if the operation
/// was exclusive or not
pub enum ExFlag {
    #[default]
    /// The operation was normal
    Normal,
    /// The operation was exclusive
    Exclusive,
}

/// The result of a Write operation on the AXI bus
pub type WriteResult = Result<(), AXI4Error>;

/// The result of a Read operation on the AXI bus
pub type ReadResult = Result<AxilData, AXI4Error>;

#[kernel]
/// Helper function to recode a [ReadResponse] into a [Result].
///
/// Because the [ReadResponse] is a bit cryptic, this function converts
/// it into a [Result].  Note that the [ResponseKind::OKAY] and [ResponseKind::EXOKAY]
/// variants are collapsed into a single [Ok] variant.
pub fn read_response_to_result(resp: ReadResponse) -> Result<AxilData, AXI4Error> {
    match resp.resp {
        response_codes::OKAY => Ok(resp.data),
        response_codes::EXOKAY => Ok(resp.data),
        response_codes::DECERR => Err(AXI4Error::DECERR),
        _ => Err(AXI4Error::SLVERR),
    }
}

#[kernel]
/// Helper function to convert a [Result] into a [ReadResponse]
///
/// This is an imperfect conversion, since the [ReadResponse] enum
/// carries multiple success variants.  For now, we are ignoring that
/// issue.  
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
/// Convert from a [Result] to a [ResponseKind] for write transactions
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
/// Convert a [ResponseKind] to a [Result] for a write transaction
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

#[derive(PartialEq, Debug, Digital, Clone, Default)]
/// MOSI signals for the read interface
///
/// These are the address, and protocol lines for the
/// interface between the master and slave device on the
/// AXI bus that implement the read side of the protocol.
/// These types are outputs for the master, and inputs
/// for the slave (hence MOSI).
pub struct ReadMOSI {
    /// Read Address
    pub araddr: AxilAddr,
    /// Read Address valid
    pub arvalid: bool,
    /// Read Data ready
    pub rready: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Default)]
/// MISO signals for the read interface
///
/// These are the protocol, data and error flags used
/// for the read interface.  This type is an input
/// for the master, and an output for the slave.
pub struct ReadMISO {
    /// Read Address ready
    pub arready: bool,
    /// Read Data
    pub rdata: AxilData,
    /// Read Data response
    pub rresp: Bits<U2>,
    /// Read Data valid
    pub rvalid: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Default)]
/// MOSI signals for the write interface
///
/// These are the protocol, data, and address lines
/// used for the write interface.  This type is an
/// output for the master, and an input for the slave.
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

#[derive(PartialEq, Debug, Clone, Digital)]
/// MISO signals for the write interface
///
/// These are protocol and response signals used
/// for the write interface.  These signals are
/// input by the master and output by the slave.
pub struct WriteMISO {
    /// Write Address ready
    pub awready: bool,
    /// Write Data ready
    pub wready: bool,
    /// Write Response
    pub bresp: Bits<U2>,
    /// Write Response valid
    pub bvalid: bool,
}

#[derive(PartialEq, Debug, Clone, Digital)]
/// All MOSI signals
///
/// These are the MOSI signals for a master
pub struct MOSI {
    /// The MOSI signals for the read bus
    pub read: ReadMOSI,
    /// The MOSI signals for the write bus
    pub write: WriteMOSI,
}

#[derive(PartialEq, Debug, Clone, Digital)]
/// All MISO signals
///
/// These are the MISO signals for a master
pub struct MISO {
    /// The MISO signals for the read bus
    pub read: ReadMISO,
    /// The MISO signals for the write bus
    pub write: WriteMISO,
}

//! AXI4Lite Read Switch
//!
//!# Purpose
//!
//! This core provides a way to connect multiple AXI Read endpoints
//! to a single Read manager.  Each AXI Read endpoint has an address
//! range that it receives.  The address decode logic is user provided
//! in the form of a pure (synthesizable) function.
//!
//!# Internals
//!
//! The switch is fairly complicated, so the internals are described in
//! pieces.  The first part is the ingest pipeline, which maps the incoming
//! requests into decoded `(port,addr)` values, and handles the case that
//! the decode logic indicates there is an error in the request.  The
//! egress pipeline is unbuffered, but includes a combinatorial circuit
//! to detect when responses have been accepted by the endpoint.
//!
#![doc = badascii!(r"
        ++RdEndpt++        ++Map+-+      ++Map++      ++Xfer++      
        |         | ?Reqst |      |?Cmd  |     |?Cmd  |  In  |?Cmd  
        |     data+------->|      +----->|     +----->|      +----> 
<+AXI+->|         | R<Req> |decode|R<Cmd>|limit|R<Cmd>|      | R<Cmd
        |      rdy|<-------+      |<-----+     |<-----+ run  |<---+ 
        +---------+        +------+      +-----+      +--+---+      
                                                         |          
        ++RdEndpt++   ++Xfer++                           v          
        |         |   |  Out |?Resp                                 
        |     resp|<--+      |<---+    <--+ From Port               
<+AXI+->|         |   |      |R<Resp>       Controller              
        |      rdy+-->|  run +---->                                 
        +---------+   +---+--+                                      
                          |                                         
                          v                                         
")]
use badascii_doc::badascii;

use rhdl::prelude::*;

use crate::{
    axi4lite::{
        core::{controller::read::ReadController, endpoint::read::ReadEndpoint},
        types::{AXI4Error, AxilAddr, ReadMISO, ReadMOSI, ReadResult},
    },
    core::dff::DFF,
    stream::{map::Map, xfer::Xfer},
};

/// The state of the switch
#[derive(PartialEq, Default, Digital, Clone, Copy)]
pub enum State {
    #[default]
    /// The switch is idle - no channel exists
    Idle,
    /// The switch has bound the endpoint to the given controller
    Bound(b4),
    /// The request was bad, and we need to reply with an error
    BadRequest,
}

/// The input address along with the port to send it to
pub type Command = Result<(b4, AxilAddr), AXI4Error>;

#[derive(Clone, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// AXI Read Switch
///
/// This core provides an AXI endpoint that can
/// fan out to multiple AXI controllers.  The address
/// decode logic must be provided by the user.  To
/// avoid excess generics, the [ReadSwitch] supports
/// a maximum of 16 controllers.  If you need more
/// then consider cascading them or writing your own.
/// The maximum pending number of transactions is
/// also limited to 255 (which is lot!).
pub struct ReadSwitch<const N: usize> {
    endpoint: ReadEndpoint,
    controllers: [ReadController; N],
    pending_count: DFF<b8>,
    state: DFF<State>,
    decode: Map<AxilAddr, Command>,
    limit: Map<Command, Command>,
    xfer_out: Xfer<ReadResult>,
    xfer_in: Xfer<Command>,
}

impl<const N: usize> ReadSwitch<N> {
    /// Create a new AXI Read Switch with the
    /// provided routing function.
    pub fn try_new<F: DigitalFn + DigitalFn2<A0 = ClockReset, A1 = AxilAddr, O = Command>>(
    ) -> Result<Self, RHDLError> {
        Ok(Self {
            endpoint: ReadEndpoint::default(),
            controllers: core::array::from_fn(|_| ReadController::default()),
            pending_count: DFF::default(),
            state: DFF::new(State::Idle),
            decode: Map::try_new::<F>()?,
            limit: Map::try_new::<range_check<N>>()?,
            xfer_out: Xfer::default(),
            xfer_in: Xfer::default(),
        })
    }
}

#[kernel]
#[doc(hidden)]
pub fn range_check<const N: usize>(_cr: ClockReset, cmd: Command) -> Command {
    match cmd {
        Ok((port, address)) => {
            if port < bits(N as u128) {
                Ok((port, address))
            } else {
                Err(AXI4Error::DECERR)
            }
        }
        Err(e) => Err(e),
    }
}

/// Input for the Read switch
#[derive(PartialEq, Clone, Copy, Digital)]
pub struct In<const N: usize> {
    /// AXI bus connection to the endpoint (subordinate interface)
    pub endpoint_0: ReadMOSI,
    /// AXI bus connection to the controllers (manager interfaces)
    pub controllers: [ReadMISO; N],
}

/// Output from the Read Switch
#[derive(PartialEq, Clone, Copy, Digital)]
pub struct Out<const N: usize> {
    /// AXI bus connection from the endpoint (subordinate interface)
    pub endpoint_0: ReadMISO,
    /// AXI bus connection from the controllers (manager interfaces)
    pub controllers: [ReadMOSI; N],
}

impl<const N: usize> SynchronousIO for ReadSwitch<N> {
    type I = In<N>;
    type O = Out<N>;
    type Kernel = kernel<N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<const N: usize>(_cr: ClockReset, i: In<N>, q: Q<N>) -> (Out<N>, D<N>) {
    let mut d = D::<N>::dont_care();
    let mut o = Out::<N>::dont_care();
    d.state = q.state;
    // Connect the endpoint AXI busses
    d.endpoint.axi = i.endpoint_0;
    o.endpoint_0 = q.endpoint.axi;
    // Connect the controller AXI busses
    for n in 0..N {
        d.controllers[n].axi = i.controllers[n];
        o.controllers[n] = q.controllers[n].axi;
    }
    // Connect the decode mapper to the endpoint
    d.decode.data = q.endpoint.req_data;
    d.endpoint.req_ready = q.decode.ready;
    // Connect the limit mapper to the decoder
    d.limit.data = q.decode.data;
    d.decode.ready = q.limit.ready;
    // Connect the xfer_in counter to the limit mapper
    d.limit.ready = q.xfer_in.ready;
    d.xfer_in.data = q.limit.data;
    // Connect the xfer out core to the endpoint
    d.endpoint.resp_data = q.xfer_out.data;
    d.xfer_out.ready = q.endpoint.resp_ready;
    // Set the ready inputs for the channel controllers to false by
    // default
    for i in 0..N {
        d.controllers[i].resp_ready.raw = false;
    }
    // Depending on the current state, the response comes from
    // one of the controllers, from the Error message, or nowhere
    match q.state {
        State::Idle => {
            d.xfer_out.data = None;
        }
        State::Bound(port) => {
            d.xfer_out.data = q.controllers[port].resp_data;
            d.controllers[port].resp_ready = q.xfer_out.ready;
        }
        State::BadRequest => {
            d.xfer_out.data = Some(Err(AXI4Error::DECERR));
        }
    }
    // Decide what to do on the input side
    // By default, all inputs are voided out
    for i in 0..N {
        d.controllers[i].req_data = None;
    }
    // Update the transaction count
    d.pending_count = if q.xfer_out.run && !q.xfer_in.run {
        q.pending_count - 1
    } else if !q.xfer_out.run && q.xfer_in.run {
        q.pending_count + 1
    } else {
        q.pending_count
    };
    // Stall the incoming pipeline
    d.xfer_in.ready.raw = false;
    match q.state {
        State::Idle => {
            if let Some(req) = q.xfer_in.data {
                // There is a request.
                if let Ok((port, _addr)) = req {
                    d.state = State::Bound(port);
                } else {
                    d.state = State::BadRequest;
                }
            }
        }
        State::Bound(port) => {
            if let Some(req) = q.xfer_in.data {
                if let Ok((req_port, addr)) = req {
                    if req_port == port {
                        d.controllers[port].req_data = Some(addr);
                        d.xfer_in.ready.raw = q.controllers[port].req_ready.raw;
                    } else if q.pending_count == 0 {
                        d.state = State::Bound(req_port);
                    }
                } else if q.pending_count == 0 {
                    d.state = State::BadRequest;
                    d.xfer_in.ready.raw = true;
                }
            }
        }
        State::BadRequest => {
            if q.xfer_out.run {
                d.state = State::Idle;
            }
        }
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use crate::{
        axi4lite::register::rom::AxiRom,
        rng::xorshift::XorShift128,
        stream::testing::{source_from_fn::SourceFromFn, utils::stalling},
    };

    use super::*;

    // ++Source++?Axil +-+Read+-+       +Switch+     ++Rom0++
    // |        | Addr |  Ctrl  |       |      | AXI |      |
    // |    data+----->|        |       |    M0|<--->|      |
    // |        |R<Req>|        |       |      |     |      |
    // |     rdy|<----+|        |       |      |     +------+
    // |        |      |        |<+Axi+>|Ep    |
    // +--------+      |        |       |      |     ++Rom1++
    //         ?resp   |        |       |      | AXI |      |
    //       <---------+        |       |    M1|<--->|      |
    //           1+--->|        |       |      |     |      |
    //             rdy +--------+       +------+     +------+
    //
    #[derive(Clone, Synchronous, SynchronousDQ)]
    #[rhdl(dq_no_prefix)]
    pub struct TestFixture {
        source: SourceFromFn<AxilAddr>,
        controller: ReadController,
        switch: ReadSwitch<2>,
        rom_0: AxiRom<4>,
        rom_1: AxiRom<4>,
    }

    impl SynchronousIO for TestFixture {
        type I = ();
        type O = Option<ReadResult>;
        type Kernel = kernel;
    }

    #[kernel]
    pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> (Option<ReadResult>, D) {
        let mut d = D::dont_care();
        d.controller.req_data = q.source;
        d.source = q.controller.req_ready;
        d.controller.resp_ready.raw = true;
        d.switch.endpoint_0 = q.controller.axi;
        d.controller.axi = q.switch.endpoint_0;
        d.rom_0.axi = q.switch.controllers[0];
        d.switch.controllers[0] = q.rom_0.read_axi;
        d.rom_1.axi = q.switch.controllers[1];
        d.switch.controllers[1] = q.rom_1.read_axi;
        let o = q.controller.resp_data;
        (o, d)
    }

    const ROM0_BASE: AxilAddr = bits(0x4_000_000);
    const ROM1_BASE: AxilAddr = bits(0x6_000_000);
    const ROM0_DATA: [b32; 4] = [
        bits(0xDEAD_BEEF),
        bits(0xBABE_FEED),
        bits(0xCAFE_1234),
        bits(0xAAAA5555),
    ];
    const ROM1_DATA: [b32; 4] = [
        bits(0x1234_5678),
        bits(0xABCD_0000),
        bits(0x5555_5555),
        bits(0xAAAA_AAAA),
    ];

    // The decode function
    #[kernel]
    pub fn decode_addr(_cr: ClockReset, req: AxilAddr) -> Command {
        let rom_0_active = req & ROM0_BASE == ROM0_BASE;
        let rom_1_active = req & ROM1_BASE == ROM1_BASE;
        match (rom_0_active, rom_1_active) {
            (true, false) => Ok((bits(0), req)),
            (true, true) => Ok((bits(1), req)),
            _ => Err(AXI4Error::DECERR),
        }
    }

    #[derive(Copy, Clone)]
    pub enum TestCase {
        Bank0(b2),
        Bank1(b2),
        Err0,
        Err1,
        ErrSwitch,
    }

    fn sim(value: TestCase) -> ReadResult {
        match value {
            TestCase::Bank0(reg) => Ok(ROM0_DATA[reg.raw() as usize]),
            TestCase::Bank1(reg) => Ok(ROM1_DATA[reg.raw() as usize]),
            _ => Err(AXI4Error::DECERR),
        }
    }

    impl From<TestCase> for AxilAddr {
        fn from(value: TestCase) -> Self {
            match value {
                TestCase::Bank0(reg) => ROM0_BASE + (reg.resize::<32>() << 2),
                TestCase::Bank1(reg) => ROM1_BASE + (reg.resize::<32>() << 2),
                TestCase::Err0 => ROM0_BASE + bits(100),
                TestCase::Err1 => ROM1_BASE + bits(100),
                TestCase::ErrSwitch => bits(0),
            }
        }
    }

    impl From<b32> for TestCase {
        fn from(value: b32) -> Self {
            let value = value.raw();
            let p = (value as f64) / (2.0_f64.powi(32));
            let reg = bits(value & 0b11);
            if p < 0.4 {
                TestCase::Bank0(reg)
            } else if p < 0.8 {
                TestCase::Bank1(reg)
            } else if p < 0.85 {
                TestCase::Err0
            } else if p < 0.90 {
                TestCase::Err1
            } else {
                TestCase::ErrSwitch
            }
        }
    }

    #[test]
    #[ignore]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let switch: ReadSwitch<2> = ReadSwitch::try_new::<decode_addr>()?;
        drc::no_combinatorial_paths(&switch)?;
        Ok(())
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let rng = XorShift128::default()
            .map(|x| b32(x as u128))
            .map(TestCase::from);
        let sink = rng.clone().map(sim);
        let source = stalling(rng.map(AxilAddr::from), 0.1);
        let uut = TestFixture {
            source: SourceFromFn::new(source),
            controller: ReadController::default(),
            switch: ReadSwitch::try_new::<decode_addr>()?,
            rom_0: AxiRom::new(ROM0_BASE, ROM0_DATA),
            rom_1: AxiRom::new(ROM1_BASE, ROM1_DATA),
        };
        let input = repeat_n((), 200).with_reset(1).clock_pos_edge(100);
        let sims = uut
            .run(input)
            .synchronous_sample()
            .filter_map(|ts| ts.output)
            .collect::<Vec<_>>();
        let sink = sink.take(sims.len()).collect::<Vec<_>>();
        assert_eq!(sink, sims);
        Ok(())
    }
}

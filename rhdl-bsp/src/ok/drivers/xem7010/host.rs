use rhdl::core::CircuitIO;
use rhdl::prelude::*;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    marker,
};
use thiserror::Error;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct WireInAddress(u8);
pub struct WireOutAddress(u8);
pub struct TriggerInAddress(u8);
pub struct TriggerOutAddress(u8);
pub struct PipeInAddress(u8);
pub struct PipeOutAddress(u8);

#[derive(Error, Debug)]
pub enum OkHostError {
    #[error("Invalid Wire In Address {0} (valid range is 0x00..0x1F)")]
    InvalidWireInAddress(u8),
    #[error("Duplicate Wire In Address {0}")]
    DuplicateWireInAddress(u8),
    #[error("Invalid Wire Out Address {0} (valid range is 0x20..0x3F)")]
    InvalidWireOutAddress(u8),
    #[error("Duplicate Wire Out Address {0}")]
    DuplicateWireOutAddress(u8),
}

#[derive(Serialize)]
pub struct Host<T> {
    marker: std::marker::PhantomData<T>,
    wire_in: BTreeMap<u8, MountPoint>,
    wire_out: BTreeMap<u8, MountPoint>,
    /*     wire_out: BTreeSet<WireOutAddress>,
       trigger_in: BTreeSet<TriggerInAddress>,
       trigger_out: BTreeSet<TriggerOutAddress>,
       pipe_in: BTreeSet<PipeInAddress>,
       pipe_out: BTreeSet<PipeOutAddress>,
    */
}

impl<T> Default for Host<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
            wire_in: Default::default(),
            wire_out: Default::default(),
        }
    }
}

struct WireIn {
    address: u8,
    bit_range: std::ops::Range<usize>,
}

fn mk_err(t: OkHostError) -> RHDLError {
    RHDLError::ExportError(ExportError::Custom(Box::new(t)))
}
/*

assign ep20wire = 16'h0000;
assign ep21wire = ep01wire + ep02wire;

// Instantiate the okHost and connect endpoints.
wire [17*2-1:0]  ok2x;
okHost okHI(
        .hi_in(hi_in), .hi_out(hi_out), .hi_inout(hi_inout), .hi_aa(hi_aa), .ti_clk(ti_clk),
        .ok1(ok1), .ok2(ok2));

okWireOR # (.N(2)) wireOR (.ok2(ok2), .ok2s(ok2x));


okWireOut    ep20 (.ok1(ok1), .ok2(ok2x[ 0*17 +: 17 ]), .ep_addr(8'h20), .ep_datain(ep20wire));
okWireOut    ep21 (.ok1(ok1), .ok2(ok2x[ 1*17 +: 17 ]), .ep_addr(8'h21), .ep_datain(ep21wire));

endmodule

*/

#[derive(Serialize)]
struct WirePoint {
    address: u8,
    mount: MountPoint,
}

#[derive(Serialize)]
struct Context {
    num_outputs: usize,
    wire_ins: Vec<WirePoint>,
    wire_outs: Vec<WirePoint>,
}

static HDL: &str = r#"
// Opal Kelly Module Interface Connections
wire        ti_clk;
wire [30:0] ok1;
wire [16:0] ok2;

assign hi_muxsel    = 1'b0;
{{ if num_outputs }} 
wire [17*{num_outputs}-1:0]  ok2x;
okWireOR # (.N({num_outputs})) wireOR (.ok2(ok2), .ok2s(ok2x));
{{ endif }}

okHost okHI(
        .hi_in(hi_in), .hi_out(hi_out), .hi_inout(hi_inout), .hi_aa(hi_aa), .ti_clk(ti_clk),
        .ok1(ok1) {{if num_outputs}}, .ok2(ok2) {{endif}} );

{{ for wire_in in wire_ins -}}
okWireIn   ok_wire_in_{@index} (.ok1(ok1), .ep_addr(8'd{wire_in.address}), .ep_dataout({wire_in.mount}));
{{ endfor }}
{{ for wire_out in wire_outs -}}
okWireOut  ok_wire_out_{@index}(.ok1(ok1), .ok2(ok2x[ {@index}*17 +: 17 ]), .ep_addr(8'd{wire_out.address}), .ep_datain({wire_out.mount}));
{{ endfor }}
"#;

impl<T: CircuitIO> Host<T> {
    pub fn wire_in(&mut self, address: u8, path: &Path) -> Result<(), RHDLError> {
        let (bits, _sub) = bit_range(<T::I as Timed>::static_kind(), path)?;
        if bits.len() != 16 {
            return Err(ExportError::SignalWidthMismatch {
                expected: 16,
                actual: bits.len(),
            }
            .into());
        }
        if address >= 0x1F {
            return Err(mk_err(OkHostError::InvalidWireInAddress(address)));
        }
        if self.wire_in.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicateWireInAddress(address)));
        }
        self.wire_in.insert(address, MountPoint::Input(bits));
        Ok(())
    }
    pub fn wire_out(&mut self, address: u8, path: &Path) -> Result<(), RHDLError> {
        let (bits, _sub) = bit_range(<T::O as Timed>::static_kind(), path)?;
        if bits.len() != 16 {
            return Err(ExportError::SignalWidthMismatch {
                expected: 16,
                actual: bits.len(),
            }
            .into());
        }
        if address < 0x20 || address > 0x3F {
            return Err(mk_err(OkHostError::InvalidWireOutAddress(address)));
        }
        if self.wire_out.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicateWireOutAddress(address)));
        }
        self.wire_out.insert(address, MountPoint::Output(bits));
        Ok(())
    }
    pub fn build(self) -> Result<Driver<T>, RHDLError> {
        let mut driver = Driver::default();
        driver.input_port("hi_in", 8);
        driver.output_port("hi_out", 2);
        driver.inout_port("hi_inout", 16);
        driver.inout_port("hi_aa", 1);
        driver.output_port("hi_muxsel", 1);
        let num_outputs = self.wire_out.len();
        let wire_ins = self
            .wire_in
            .into_iter()
            .map(|(addr, mount)| WirePoint {
                address: addr,
                mount,
            })
            .collect();
        let wire_outs = self
            .wire_out
            .into_iter()
            .map(|(addr, mount)| WirePoint {
                address: addr,
                mount,
            })
            .collect();
        let context = Context {
            num_outputs,
            wire_ins,
            wire_outs,
        };
        driver.render_hdl(HDL, &context)?;
        Ok(driver)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use rhdl::prelude::*;

    #[test]
    fn test_host() -> Result<(), RHDLError> {
        #[derive(PartialEq, Digital, Timed)]
        struct O {
            out1: Signal<b16, Red>,
            out2: Signal<b16, Red>,
        }

        #[derive(PartialEq, Digital, Timed)]
        struct I {
            in1: Signal<b16, Red>,
            in2: Signal<b16, Red>,
        }

        #[derive(Clone)]
        struct U;

        impl CircuitDQ for U {
            type D = ();
            type Q = ();
        }

        impl CircuitIO for U {
            type I = I;
            type O = O;
            type Kernel = NoKernel2<I, (), (O, ())>;
        }

        let mut ok_host = super::Host::<U>::default();
        ok_host.wire_in(0, &path!(.in1))?;
        ok_host.wire_in(2, &path!(.in2))?;
        ok_host.wire_out(0x20, &path!(.out1))?;
        ok_host.wire_out(0x24, &path!(.out2))?;
        let driver = ok_host.build()?;
        let expect = expect_file!("ok_host.expect");
        expect.assert_eq(&driver.hdl);
        Ok(())
    }
}

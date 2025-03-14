use rhdl::prelude::*;
use rhdl::{core::CircuitIO, rtt::TraceType};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    marker,
};
use thiserror::Error;

use crate::drivers::{get_clock_input, get_clock_output, get_untyped_input, get_untyped_output};

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
    #[error("Invalid Trigger In Address {0} (valid range is 0x40..0x5F)")]
    InvalidTriggerInAddress(u8),
    #[error("Duplicate Trigger In Address {0}")]
    DuplicateTriggerInAddress(u8),
    #[error("Invalid Trigger Out Address {0} (valid range is 0x60..0x7F)")]
    InvalidTriggerOutAddress(u8),
    #[error("Duplicate Trigger Out Address {0}")]
    DuplicateTriggerOutAddress(u8),
}

#[derive(Serialize)]
pub struct Host<T> {
    marker: std::marker::PhantomData<T>,
    wire_in: BTreeMap<u8, MountPoint>,
    wire_out: BTreeMap<u8, MountPoint>,
    trigger_in: BTreeMap<u8, TriggerPoint>,
    trigger_out: BTreeMap<u8, TriggerPoint>,
    /*
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
            trigger_in: Default::default(),
            trigger_out: Default::default(),
        }
    }
}

fn mk_err(t: OkHostError) -> RHDLError {
    RHDLError::ExportError(ExportError::Custom(Box::new(t)))
}

#[derive(Serialize)]
struct WirePoint {
    address: u8,
    mount: MountPoint,
}

#[derive(Serialize)]
struct TriggerPoint {
    address: u8,
    clock: MountPoint,
    triggers: MountPoint,
}

#[derive(Serialize)]
struct Context {
    num_outputs: usize,
    wire_ins: Vec<WirePoint>,
    wire_outs: Vec<(usize, WirePoint)>,
    trigger_ins: Vec<TriggerPoint>,
    trigger_outs: Vec<(usize, TriggerPoint)>,
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
okWireIn   ok_wire_in_{@index} (
   .ok1(ok1), 
   .ep_addr(8'd{wire_in.address}), 
   .ep_dataout({wire_in.mount})
);
{{ endfor }}
{{ for wire_out in wire_outs -}}
okWireOut  ok_wire_out_{@index} (
    .ok1(ok1), 
    .ok2(ok2x[ {wire_out.0}*17 +: 17 ]), 
    .ep_addr(8'd{wire_out.1.address}), 
    .ep_datain({wire_out.1.mount})
);
{{ endfor }}
{{ for trigger_in in trigger_ins -}}
okTriggerIn ok_trigger_in{@index} (
    .ok1(ok1), 
    .ep_addr(8'd{trigger_in.address}), 
    .ep_clk({trigger_in.clock})
    .ep_trigger({trigger_in.triggers})
);
{{ endfor }}
{{ for trigger_out in trigger_outs -}}
okTriggerOut ok_trigger_out{@index} (
    .ok1(ok1),
    .ok2(ok2x[ {trigger_out.0}*17 +: 17 ]), 
    .ep_addr(8'd{trigger_out.1.address}),
    .ep_clk({trigger_out.1.clock})
    .ep_trigger({trigger_out.1.triggers})
);
{{ endfor }}
"#;

impl<T: CircuitIO> Host<T> {
    pub fn wire_in(&mut self, address: u8, path: &Path) -> Result<(), RHDLError> {
        let value = get_untyped_input::<T>(path, 16)?;
        if !(0x00..=0x1F).contains(&address) {
            return Err(mk_err(OkHostError::InvalidWireInAddress(address)));
        }
        if self.wire_in.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicateWireInAddress(address)));
        }
        self.wire_in.insert(address, value);
        Ok(())
    }
    pub fn wire_out(&mut self, address: u8, path: &Path) -> Result<(), RHDLError> {
        let value = get_untyped_output::<T>(path, 16)?;
        if !(0x20..=0x3F).contains(&address) {
            return Err(mk_err(OkHostError::InvalidWireOutAddress(address)));
        }
        if self.wire_out.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicateWireOutAddress(address)));
        }
        self.wire_out.insert(address, value);
        Ok(())
    }
    pub fn trigger_in(
        &mut self,
        address: u8,
        clock_path: &Path,
        trigger_path: &Path,
    ) -> Result<(), RHDLError> {
        // We need an input clock
        let clock = get_clock_output::<T>(clock_path)?;
        let triggers = get_untyped_input::<T>(trigger_path, 16)?;
        if !(0x40..=0x5F).contains(&address) {
            return Err(mk_err(OkHostError::InvalidTriggerInAddress(address)));
        }
        if self.trigger_in.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicateTriggerInAddress(address)));
        }
        self.trigger_in.insert(
            address,
            TriggerPoint {
                address,
                clock,
                triggers,
            },
        );
        Ok(())
    }
    pub fn trigger_out(
        &mut self,
        address: u8,
        clock_path: &Path,
        trigger_path: &Path,
    ) -> Result<(), RHDLError> {
        let clock = get_clock_output::<T>(clock_path)?;
        let triggers = get_untyped_output::<T>(trigger_path, 16)?;
        if !(0x60..=0x7F).contains(&address) {
            return Err(mk_err(OkHostError::InvalidTriggerOutAddress(address)));
        }
        if self.trigger_out.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicateTriggerOutAddress(address)));
        }
        self.trigger_out.insert(
            address,
            TriggerPoint {
                address,
                clock,
                triggers,
            },
        );
        Ok(())
    }
    pub fn build(self) -> Result<Driver<T>, RHDLError> {
        let mut driver = Driver::default();
        driver.input_port("hi_in", 8);
        driver.output_port("hi_out", 2);
        driver.inout_port("hi_inout", 16);
        driver.inout_port("hi_aa", 1);
        driver.output_port("hi_muxsel", 1);
        let wire_ins = self
            .wire_in
            .into_iter()
            .map(|(addr, mount)| WirePoint {
                address: addr,
                mount,
            })
            .collect();
        let mut output_counter = 0;
        let wire_outs: Vec<(usize, WirePoint)> = self
            .wire_out
            .into_iter()
            .map(|(addr, mount)| {
                let out = output_counter;
                output_counter += 1;
                (
                    out,
                    WirePoint {
                        address: addr,
                        mount,
                    },
                )
            })
            .collect();
        let trigger_ins = self.trigger_in.into_values().collect();
        let trigger_outs: Vec<(usize, TriggerPoint)> = self
            .trigger_out
            .into_values()
            .map(|x| {
                let out = output_counter;
                output_counter += 1;
                (out, x)
            })
            .collect();
        let num_outputs = wire_outs.len() + trigger_outs.len();
        let context = Context {
            num_outputs,
            wire_ins,
            wire_outs,
            trigger_ins,
            trigger_outs,
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
            t_clk: Signal<Clock, Red>,
            t_out: Signal<b16, Red>,
        }

        #[derive(PartialEq, Digital, Timed)]
        struct I {
            in1: Signal<b16, Red>,
            in2: Signal<b16, Red>,
            t_in: Signal<b16, Red>,
            t_in2: Signal<b16, Red>,
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
        ok_host.trigger_in(0x42, &path!(.t_clk.val()), &path!(.t_in))?;
        ok_host.trigger_in(0x40, &path!(.t_clk.val()), &path!(.t_in2))?;
        ok_host.trigger_out(0x60, &path!(.t_clk.val()), &path!(.t_out))?;
        let driver = ok_host.build()?;
        let expect = expect_file!("ok_host.expect");
        expect.assert_eq(&driver.hdl);
        Ok(())
    }
}

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rhdl::core::CircuitIO;
use rhdl::prelude::*;
use std::collections::BTreeMap;
use thiserror::Error;

use crate::drivers::{get_clock_output, get_untyped_input, get_untyped_output};

#[derive(Error, Debug)]
pub enum OkHostError {
    #[error("Invalid Wire In Address {0} (valid range is 0x00..0x20)")]
    InvalidWireInAddress(u8),
    #[error("Duplicate Wire In Address {0}")]
    DuplicateWireInAddress(u8),
    #[error("Invalid Wire Out Address {0} (valid range is 0x20..0x40)")]
    InvalidWireOutAddress(u8),
    #[error("Duplicate Wire Out Address {0}")]
    DuplicateWireOutAddress(u8),
    #[error("Invalid Trigger In Address {0} (valid range is 0x40..0x60)")]
    InvalidTriggerInAddress(u8),
    #[error("Duplicate Trigger In Address {0}")]
    DuplicateTriggerInAddress(u8),
    #[error("Invalid Trigger Out Address {0} (valid range is 0x60..0x80)")]
    InvalidTriggerOutAddress(u8),
    #[error("Duplicate Trigger Out Address {0}")]
    DuplicateTriggerOutAddress(u8),
    #[error("Invalid Pipe In Address {0} (valid range is 0x80..0xA0")]
    InvalidPipeInAddress(u8),
    #[error("Duplicate Pipe In Address {0}")]
    DuplicatePipeInAddress(u8),
    #[error("Invalid Pipe Out Address {0} (valid range is 0xA0..0xC0")]
    InvalidPipeOutAddress(u8),
    #[error("Duplicate Pipe Out Address {0}")]
    DuplicatePipeOutAddress(u8),
}

const WIRE_IN_ADDRESS_RANGE: std::ops::Range<u8> = 0x00..0x20;
const WIRE_OUT_ADDRESS_RANGE: std::ops::Range<u8> = 0x20..0x40;
const TRIGGER_IN_ADDRESS_RANGE: std::ops::Range<u8> = 0x40..0x60;
const TRIGGER_OUT_ADDRESS_RANGE: std::ops::Range<u8> = 0x60..0x80;
const PIPE_IN_ADDRESS_RANGE: std::ops::Range<u8> = 0x80..0xA0;
const PIPE_OUT_ADDRESS_RANGE: std::ops::Range<u8> = 0xA0..0xC0;

pub struct Host<T> {
    marker: std::marker::PhantomData<T>,
    wire_in: BTreeMap<u8, MountPoint>,
    wire_out: BTreeMap<u8, MountPoint>,
    trigger_in: BTreeMap<u8, TriggerPoint>,
    trigger_out: BTreeMap<u8, TriggerPoint>,
    pipe_in: BTreeMap<u8, PipePoint>,
    pipe_out: BTreeMap<u8, PipePoint>,
    bt_pipe_in: BTreeMap<u8, BTPipePoint>,
    bt_pipe_out: BTreeMap<u8, BTPipePoint>,
}

impl<T> Default for Host<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
            wire_in: Default::default(),
            wire_out: Default::default(),
            trigger_in: Default::default(),
            trigger_out: Default::default(),
            pipe_in: Default::default(),
            pipe_out: Default::default(),
            bt_pipe_in: Default::default(),
            bt_pipe_out: Default::default(),
        }
    }
}

fn mk_err(t: OkHostError) -> RHDLError {
    RHDLError::ExportError(ExportError::Custom(t.into()))
}

struct WirePoint {
    address: u8,
    mount: MountPoint,
}

struct TriggerPoint {
    address: u8,
    clock: MountPoint,
    triggers: MountPoint,
}

struct PipePoint {
    address: u8,
    data_mount: MountPoint,
    flag_mount: MountPoint,
}

struct BTPipePoint {
    address: u8,
    data_mount: MountPoint,
    flag_mount: MountPoint,
    ready_mount: MountPoint,
    strobe_mount: MountPoint,
}

fn tag_with_output_slot<S>(output_counter: &mut usize, data: BTreeMap<u8, S>) -> Vec<(usize, S)> {
    data.into_values()
        .map(|x| {
            let out = *output_counter;
            *output_counter += 1;
            (out, x)
        })
        .collect()
}

fn lit_address(address: u8) -> vlog::LitVerilog {
    vlog::lit_verilog(8, &format!("d{address}"))
}

fn wire_in(points: &[WirePoint]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().enumerate().map(|(index, point)| {
        let addr = lit_address(point.address);
        let mount = &point.mount;
        let name = format_ident!("ok_wire_in_{index}");
        quote! {
            okWireIn  #name (
               .ok1(ok1),
               .ep_addr(#addr),
               .ep_dataout(#mount)
            );
        }
    })
}

fn wire_out(points: &[(usize, WirePoint)]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().map(|(out, point)| {
        let addr = lit_address(point.address);
        let mount = &point.mount;
        let name = format_ident!("ok_wire_out_{out}");
        let out = syn::Index::from(*out * 17);
        quote! {
            okWireOut  #name (
                .ok1(ok1),
                .ok2(ok2x[ #out +: 17 ]),
                .ep_addr(#addr),
                .ep_datain(#mount)
            );
        }
    })
}

fn trigger_in(points: &[TriggerPoint]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().enumerate().map(|(index, point)| {
        let addr = lit_address(point.address);
        let clock = &point.clock;
        let triggers = &point.triggers;
        let name = format_ident!("ok_trigger_in{index}");
        quote! {
            okTriggerIn #name (
                .ok1(ok1),
                .ep_addr(#addr),
                .ep_clk(#clock),
                .ep_trigger(#triggers)
            );
        }
    })
}

fn trigger_out(points: &[(usize, TriggerPoint)]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().map(|(out, point)| {
        let addr = lit_address(point.address);
        let clock = &point.clock;
        let triggers = &point.triggers;
        let name = format_ident!("ok_trigger_out{out}");
        let out = syn::Index::from(*out * 17);
        quote! {
            okTriggerOut #name (
                .ok1(ok1),
                .ok2(ok2x[ #out +: 17 ]),
                .ep_addr(#addr),
                .ep_clk(#clock),
                .ep_trigger(#triggers)
            );
        }
    })
}

fn pipe_in(points: &[(usize, PipePoint)]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().map(|(out, point)| {
        let addr = lit_address(point.address);
        let data_mount = &point.data_mount;
        let flag_mount = &point.flag_mount;
        let name = format_ident!("ok_pipe_in{out}");
        let out = syn::Index::from(*out * 17);
        quote! {
            okPipeIn #name (
                .ok1(ok1),
                .ok2(ok2x[ #out +: 17 ]),
                .ep_addr(#addr),
                .ep_dataout(#data_mount),
                .ep_write(#flag_mount),
            );
        }
    })
}

fn pipe_out(points: &[(usize, PipePoint)]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().map(|(out, point)| {
        let addr = lit_address(point.address);
        let data_mount = &point.data_mount;
        let flag_mount = &point.flag_mount;
        let name = format_ident!("ok_pipe_out{out}");
        let out = syn::Index::from(*out * 17);
        quote! {
            okPipeOut #name (
                .ok1(ok1),
                .ok2(ok2x[ #out +: 17 ]),
                .ep_addr(#addr),
                .ep_datain(#data_mount),
                .ep_read(#flag_mount),
            );
        }
    })
}

fn bt_pipe_in(points: &[(usize, BTPipePoint)]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().map(|(out, point)| {
        let addr = lit_address(point.address);
        let data_mount = &point.data_mount;
        let flag_mount = &point.flag_mount;
        let ready_mount = &point.ready_mount;
        let strobe_mount = &point.strobe_mount;
        let name = format_ident!("ok_bt_pipe_in{out}");
        let out = syn::Index::from(*out * 17);
        quote! {
            okBTPipeIn #name (
                .ok1(ok1),
                .ok2(ok2x[ #out +: 17]),
                .ep_addr(#addr),
                .ep_dataout(#data_mount),
                .ep_write(#flag_mount),
                .ep_blockstrobe(#strobe_mount),
                .ep_ready(#ready_mount),
            );
        }
    })
}

fn bt_pipe_out(points: &[(usize, BTPipePoint)]) -> impl Iterator<Item = TokenStream> + '_ {
    points.iter().map(|(out, point)| {
        let addr = lit_address(point.address);
        let data_mount = &point.data_mount;
        let flag_mount = &point.flag_mount;
        let ready_mount = &point.ready_mount;
        let strobe_mount = &point.strobe_mount;
        let name = format_ident!("ok_bt_pipe_out{out}");
        let out = syn::Index::from(*out * 17);
        quote! {
            okBTPipeOut #name (
                .ok1(ok1),
                .ok2(ok2x[ #out +: 17]),
                .ep_addr(#addr),
                .ep_datain(#data_mount),
                .ep_read(#flag_mount),
                .ep_blockstrobe(#strobe_mount),
                .ep_ready(#ready_mount),
            );
        }
    })
}

impl<T: CircuitIO> Host<T> {
    pub fn wire_in(&mut self, address: u8, path: &Path) -> Result<(), RHDLError> {
        let value = get_untyped_input::<T>(path, 16)?;
        if !WIRE_IN_ADDRESS_RANGE.contains(&address) {
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
        if !WIRE_OUT_ADDRESS_RANGE.contains(&address) {
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
        if !TRIGGER_IN_ADDRESS_RANGE.contains(&address) {
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
        if !TRIGGER_OUT_ADDRESS_RANGE.contains(&address) {
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
    pub fn pipe_in(
        &mut self,
        address: u8,
        data_path: &Path,
        write_flag_path: &Path,
    ) -> Result<(), RHDLError> {
        let data_mount = get_untyped_input::<T>(data_path, 16)?;
        let flag_mount = get_untyped_input::<T>(write_flag_path, 1)?;
        if !PIPE_IN_ADDRESS_RANGE.contains(&address) {
            return Err(mk_err(OkHostError::InvalidPipeInAddress(address)));
        }
        if self.pipe_in.contains_key(&address) || self.bt_pipe_in.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicatePipeInAddress(address)));
        }
        self.pipe_in.insert(
            address,
            PipePoint {
                address,
                data_mount,
                flag_mount,
            },
        );
        Ok(())
    }
    pub fn bt_pipe_in(
        &mut self,
        address: u8,
        data_path: &Path,
        ready_path: &Path,
        strobe_path: &Path,
        write_flag_path: &Path,
    ) -> Result<(), RHDLError> {
        let data_mount = get_untyped_input::<T>(data_path, 16)?;
        let ready_mount = get_untyped_output::<T>(ready_path, 1)?;
        let strobe_mount = get_untyped_input::<T>(strobe_path, 1)?;
        let flag_mount = get_untyped_input::<T>(write_flag_path, 1)?;
        if !PIPE_IN_ADDRESS_RANGE.contains(&address) {
            return Err(mk_err(OkHostError::InvalidPipeInAddress(address)));
        }
        if self.pipe_in.contains_key(&address) || self.bt_pipe_in.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicatePipeInAddress(address)));
        }
        self.bt_pipe_in.insert(
            address,
            BTPipePoint {
                address,
                data_mount,
                flag_mount,
                ready_mount,
                strobe_mount,
            },
        );
        Ok(())
    }
    pub fn pipe_out(
        &mut self,
        address: u8,
        data_path: &Path,
        read_next_path: &Path,
    ) -> Result<(), RHDLError> {
        let data_mount = get_untyped_output::<T>(data_path, 16)?;
        let flag_mount = get_untyped_input::<T>(read_next_path, 1)?;
        if !PIPE_OUT_ADDRESS_RANGE.contains(&address) {
            return Err(mk_err(OkHostError::InvalidPipeOutAddress(address)));
        }
        if self.pipe_out.contains_key(&address) || self.bt_pipe_out.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicatePipeOutAddress(address)));
        }
        self.pipe_out.insert(
            address,
            PipePoint {
                address,
                data_mount,
                flag_mount,
            },
        );
        Ok(())
    }
    pub fn bt_pipe_out(
        &mut self,
        address: u8,
        data_path: &Path,
        ready_path: &Path,
        strobe_path: &Path,
        read_next_path: &Path,
    ) -> Result<(), RHDLError> {
        let data_mount = get_untyped_output::<T>(data_path, 16)?;
        let ready_mount = get_untyped_output::<T>(ready_path, 1)?;
        let strobe_mount = get_untyped_input::<T>(strobe_path, 1)?;
        let flag_mount = get_untyped_input::<T>(read_next_path, 1)?;
        if !PIPE_OUT_ADDRESS_RANGE.contains(&address) {
            return Err(mk_err(OkHostError::InvalidPipeOutAddress(address)));
        }
        if self.pipe_out.contains_key(&address) || self.bt_pipe_out.contains_key(&address) {
            return Err(mk_err(OkHostError::DuplicatePipeOutAddress(address)));
        }
        self.bt_pipe_out.insert(
            address,
            BTPipePoint {
                address,
                data_mount,
                flag_mount,
                ready_mount,
                strobe_mount,
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
            .collect::<Vec<_>>();
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
            .collect::<Vec<_>>();
        let trigger_ins = self.trigger_in.into_values().collect::<Vec<_>>();
        let trigger_outs = tag_with_output_slot(&mut output_counter, self.trigger_out);
        let pipe_ins = tag_with_output_slot(&mut output_counter, self.pipe_in);
        let pipe_outs: Vec<(usize, PipePoint)> =
            tag_with_output_slot(&mut output_counter, self.pipe_out);
        let bt_pipe_ins: Vec<(usize, BTPipePoint)> =
            tag_with_output_slot(&mut output_counter, self.bt_pipe_in);
        let bt_pipe_outs: Vec<(usize, BTPipePoint)> =
            tag_with_output_slot(&mut output_counter, self.bt_pipe_out);
        let num_outputs = wire_outs.len()
            + trigger_outs.len()
            + pipe_ins.len()
            + pipe_outs.len()
            + bt_pipe_ins.len()
            + bt_pipe_outs.len();
        let preamble = if num_outputs > 0 {
            let ok2range: vlog::BitRange = (0..(17 * num_outputs)).into();
            let num_outputs = syn::Index::from(num_outputs);
            quote! {
                wire [#ok2range]  ok2x;
                okWireOR # (.N(#num_outputs)) wireOR (.ok2(ok2), .ok2s(ok2x));
            }
        } else {
            quote! {}
        };
        let ok2_port = if num_outputs > 0 {
            quote! {, .ok2(ok2) }
        } else {
            quote! {}
        };
        let wire_ins = wire_in(&wire_ins);
        let wire_outs = wire_out(&wire_outs);
        let trigger_ins = trigger_in(&trigger_ins);
        let trigger_outs = trigger_out(&trigger_outs);
        let pipe_ins = pipe_in(&pipe_ins);
        let pipe_outs = pipe_out(&pipe_outs);
        let bt_pipe_ins = bt_pipe_in(&bt_pipe_ins);
        let bt_pipe_outs = bt_pipe_out(&bt_pipe_outs);
        driver.hdl = parse_quote_miette! {
            // Opal Kelly Module Interface Connections
            wire        ti_clk;
            wire [30:0] ok1;
            wire [16:0] ok2;

            assign hi_muxsel    = 1'b0;
            #preamble
            okHost okHI(
                .hi_in(hi_in), .hi_out(hi_out), .hi_inout(hi_inout), .hi_aa(hi_aa), .ti_clk(ti_clk),
                .ok1(ok1) #ok2_port );
            #(#wire_ins)*
            #(#wire_outs)*
            #(#trigger_ins)*
            #(#trigger_outs)*
            #(#pipe_ins)*
            #(#pipe_outs)*
            #(#bt_pipe_ins)*
            #(#bt_pipe_outs)*
        }?;
        Ok(driver)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use rhdl::prelude::*;

    #[test]
    fn test_host() -> miette::Result<()> {
        #[derive(PartialEq, Digital, Timed)]
        struct O {
            out1: Signal<b16, Red>,
            out2: Signal<b16, Red>,
            t_clk: Signal<Clock, Red>,
            t_out: Signal<b16, Red>,
            p_out: Signal<b16, Red>,
            bt_ready: Signal<bool, Red>,
        }

        #[derive(PartialEq, Digital, Timed)]
        struct I {
            in1: Signal<b16, Red>,
            in2: Signal<b16, Red>,
            t_in: Signal<b16, Red>,
            t_in2: Signal<b16, Red>,
            p_in: Signal<b16, Red>,
            p_in_write: Signal<bool, Red>,
            p_out_read: Signal<bool, Red>,
            bt_strobe: Signal<bool, Red>,
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
        ok_host.pipe_in(0x80, &path!(.p_in), &path!(.p_in_write))?;
        ok_host.pipe_out(0xA0, &path!(.p_out), &path!(.p_out_read))?;
        ok_host.bt_pipe_in(
            0x81,
            &path!(.p_in),
            &path!(.bt_ready),
            &path!(.bt_strobe),
            &path!(.p_in_write),
        )?;
        ok_host.bt_pipe_out(
            0xA1,
            &path!(.p_out),
            &path!(.bt_ready),
            &path!(.bt_strobe),
            &path!(.p_out_read),
        )?;
        let driver = ok_host.build()?;
        let expect = expect_file!("ok_host.expect");
        expect.assert_eq(&driver.hdl.pretty());
        Ok(())
    }
}

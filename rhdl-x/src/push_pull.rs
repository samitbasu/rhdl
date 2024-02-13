use anyhow::ensure;
use rhdl_bits::alias::*;
use rhdl_bits::Bits;
use rhdl_core::note;
use rhdl_core::note_init_db;
use rhdl_core::note_take;
use rhdl_core::note_time;
use rhdl_core::Digital;
use rhdl_core::DigitalFn;
use rhdl_core::Notable;
use rhdl_core::NoteKey;
use rhdl_core::NoteWriter;
use rhdl_macro::kernel;
use rhdl_macro::Digital;

use crate::circuit::root_descriptor;
use crate::circuit::root_hdl;
use crate::circuit::BufZ;
use crate::circuit::HDLDescriptor;
use crate::circuit::Tristate;
use crate::clock::Clock;
use crate::dff::DFFI;
use crate::strobe::StrobeI;
use crate::{circuit::Circuit, constant::Constant, dff::DFF, strobe::Strobe};

#[derive(Default, Clone)]
pub struct ZDriver<T: Digital> {
    phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct ZDriverI<T: Digital> {
    pub data: T,
    pub enable: bool,
}

impl<T: Digital + Tristate> DigitalFn for ZDriver<T> {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        todo!()
    }
}

impl<T: Digital + Tristate> ZDriver<T> {
    fn as_verilog(&self) -> crate::circuit::HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let input_bits = T::bits();
        let output_bits = T::bits().saturating_sub(1);
        let io_bits = T::bits().saturating_sub(1);
        let code = format!(
            "
module {module_name}(input wire[{input_bits}:0] i, output wire[{output_bits}:0], inout wire[{io_bits}:0] io);   
 assign enable = i[{input_bits}];
 assign o = io;
 assign io = enable ? i : {input_bits}'bz;
 endmodule
            "
        );
        HDLDescriptor {
            name: module_name,
            body: code,
            children: Default::default(),
        }
    }
}

impl<T: Digital + Tristate> Circuit for ZDriver<T> {
    type I = ZDriverI<T>;

    type O = T;

    type IO = T;

    type Q = ();

    type D = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |i, _| (i.data, ());

    type S = ();

    fn sim(&self, input: Self::I, io: Self::IO, state: &mut Self::S) -> (Self::O, BufZ<T>) {
        if input.enable {
            (
                input.data,
                BufZ::<T> {
                    value: input.data,
                    mask: <T as Tristate>::ENABLED,
                },
            )
        } else {
            (
                io,
                BufZ::<T> {
                    value: io,
                    mask: <T as Tristate>::DISABLED,
                },
            )
        }
    }

    fn name(&self) -> &'static str {
        "ZDriver"
    }

    fn as_hdl(
        &self,
        kind: crate::circuit::HDLKind,
    ) -> anyhow::Result<crate::circuit::HDLDescriptor> {
        ensure!(kind == crate::circuit::HDLKind::Verilog);
        Ok(self.as_verilog())
    }

    fn descriptor(&self) -> crate::circuit::CircuitDescriptor {
        crate::circuit::root_descriptor(self)
    }
}

#[derive(Clone)]
pub struct Push {
    strobe: Strobe<32>,
    value: Constant<Bits<8>>,
    buf_z: ZDriver<Bits<8>>,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushQ {
    strobe: <Strobe<32> as Circuit>::O,
    value: <Constant<Bits<8>> as Circuit>::O,
    buf_z: <ZDriver<Bits<8>> as Circuit>::O,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushD {
    strobe: <Strobe<32> as Circuit>::I,
    value: <Constant<Bits<8>> as Circuit>::I,
    buf_z: <ZDriver<Bits<8>> as Circuit>::I,
}

impl<const N: usize> Tristate for Bits<N> {
    type Mask = Bits<N>;
    const ENABLED: Self::Mask = Bits::<N>::MASK;
    const DISABLED: Self::Mask = Bits::<N>::ZERO;
}

impl<const N: usize> Notable for BufZ<Bits<N>> {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_tristate(key, self.value.0, self.mask.0, N as u8);
    }
}

impl Circuit for Push {
    type I = Clock;

    type O = ();

    type IO = b8;

    type Q = PushQ;

    type D = PushD;

    type S = (
        Self::Q,
        <Strobe<32> as Circuit>::S,
        <Constant<Bits<8>> as Circuit>::S,
        <ZDriver<Bits<8>> as Circuit>::S,
    );

    type Update = pushd;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = pushd;

    fn name(&self) -> &'static str {
        "PushD"
    }

    fn descriptor(&self) -> crate::circuit::CircuitDescriptor {
        let mut ret = root_descriptor(self);
        ret.children
            .insert("strobe".to_string(), self.strobe.descriptor());
        ret.children
            .insert("value".to_string(), self.value.descriptor());
        ret.children
            .insert("buf_z".to_string(), self.buf_z.descriptor());
        ret
    }

    fn as_hdl(&self, kind: crate::circuit::HDLKind) -> anyhow::Result<HDLDescriptor> {
        let mut ret = root_hdl(self, kind)?;
        ret.children
            .insert("strobe".to_string(), self.strobe.as_hdl(kind)?);
        ret.children
            .insert("value".to_string(), self.value.as_hdl(kind)?);
        ret.children
            .insert("buf_z".to_string(), self.buf_z.as_hdl(kind)?);
        Ok(ret)
    }

    // TODO - figure out how to handle splitting of the bufz across children
    fn sim(&self, input: Self::I, io: Self::IO, state: &mut Self::S) -> (Self::O, BufZ<Self::IO>) {
        loop {
            let mut bufz = Default::default();
            let prev_state = state.clone();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            (state.0.strobe, _) = self.strobe.sim(internal_inputs.strobe, (), &mut state.1);
            (state.0.value, _) = self.value.sim(internal_inputs.value, (), &mut state.2);
            (state.0.buf_z, bufz) = self.buf_z.sim(internal_inputs.buf_z, io, &mut state.3);
            if state == &prev_state {
                return ((), bufz);
            }
        }
    }
}

#[kernel]
pub fn pushd(i: Clock, q: PushQ) -> ((), PushD) {
    let mut d = PushD::default();
    d.buf_z.data = q.value;
    d.buf_z.enable = q.strobe;
    d.strobe.clock = i;
    d.strobe.enable = true;
    note("d", d);
    note("q", q);
    ((), d)
}

#[test]
fn test_push_as_verilog() {
    let push = Push {
        strobe: Strobe::new(b32(5)),
        value: Constant::from(b8(5)),
        buf_z: ZDriver::default(),
    };
    let top = push.as_hdl(crate::circuit::HDLKind::Verilog).unwrap();
    println!("{}", top);
}

#[test]
fn test_simulate_push() {
    let push = Push {
        strobe: Strobe::new(b32(5)),
        value: Constant::from(b8(5)),
        buf_z: ZDriver::default(),
    };
    let mut state = push.init_state();
    note_init_db();
    note_time(0);
    for (ndx, input) in crate::clock::clock().take(500).enumerate() {
        note_time(ndx as u64 * 100);
        let mut z_state = BufZ::<b8>::default();
        note("clock", input);
        loop {
            let ((), bufz) = push.sim(input, z_state.value, &mut state);
            if bufz == z_state {
                break;
            }
            z_state = bufz;
        }
        note("bus", z_state);
    }
    let db = note_take().unwrap();
    let push = std::fs::File::create("push.vcd").unwrap();
    db.dump_vcd(&[], push).unwrap();
}

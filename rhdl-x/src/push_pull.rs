use anyhow::ensure;
use rhdl_bits::alias::*;
use rhdl_bits::Bits;
use rhdl_core::note;
use rhdl_core::note_init_db;
use rhdl_core::note_pop_path;
use rhdl_core::note_push_path;
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
use crate::circuit::TristateBuf;
use crate::clock::Clock;
use crate::dff::DFF;
use crate::{circuit::Circuit, constant::Constant, strobe::Strobe};

#[derive(Default, Clone)]
pub struct ZDriver<const N: usize> {}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct ZDriverI<const N: usize> {
    pub data: Bits<N>,
    pub enable: bool,
}

impl<const N: usize> DigitalFn for ZDriver<N> {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        todo!()
    }
}

impl<const N: usize> ZDriver<N> {
    fn as_verilog(&self) -> crate::circuit::HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let input_bits = N;
        let output_bits = N.saturating_sub(1);
        let io_bits = N.saturating_sub(1);
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

impl<const N: usize> Circuit for ZDriver<N> {
    type I = ZDriverI<N>;

    type O = Bits<N>;

    type Q = ();

    type D = ();

    const NumZ: usize = N;

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |i, _| (i.data, ());

    type S = ();

    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut BufZ) -> Self::O {
        if input.enable {
            io.drive(input.data);
        } else {
            io.tri_state::<N>();
        }
        io.read()
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

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub enum Side {
    #[default]
    Left,
    Right,
}

#[derive(Clone)]
pub struct Push {
    strobe: Strobe<32>,
    value: Constant<Bits<8>>,
    buf_z: ZDriver<8>,
    side: DFF<Side>,
    latch: DFF<Bits<8>>,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushQ {
    strobe: <Strobe<32> as Circuit>::O,
    value: <Constant<Bits<8>> as Circuit>::O,
    buf_z: <ZDriver<8> as Circuit>::O,
    side: <DFF<Side> as Circuit>::O,
    latch: <DFF<Bits<8>> as Circuit>::O,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushD {
    strobe: <Strobe<32> as Circuit>::I,
    value: <Constant<Bits<8>> as Circuit>::I,
    buf_z: <ZDriver<8> as Circuit>::I,
    side: <DFF<Side> as Circuit>::I,
    latch: <DFF<Bits<8>> as Circuit>::I,
}

/*
impl<const N: usize> Notable for BufZ<Bits<N>> {
    fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
        writer.write_tristate(key, self.value.0, self.mask.0, N as u8);
    }
}
*/

impl Circuit for Push {
    type I = Clock;

    type O = b8;

    const NumZ: usize = 8;

    type Q = PushQ;

    type D = PushD;

    type S = (
        Self::Q,
        <Strobe<32> as Circuit>::S,
        <Constant<Bits<8>> as Circuit>::S,
        <ZDriver<8> as Circuit>::S,
        <DFF<Side> as Circuit>::S,
        <DFF<Bits<8>> as Circuit>::S,
    );

    fn init_state(&self) -> Self::S {
        (
            Default::default(),
            self.strobe.init_state(),
            self.value.init_state(),
            self.buf_z.init_state(),
            self.side.init_state(),
            self.latch.init_state(),
        )
    }

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
        ret.children
            .insert("side".to_string(), self.side.descriptor());
        ret.children
            .insert("latch".to_string(), self.latch.descriptor());
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
        ret.children
            .insert("side".to_string(), self.side.as_hdl(kind)?);
        ret.children
            .insert("latch".to_string(), self.latch.as_hdl(kind)?);
        Ok(ret)
    }

    // TODO - figure out how to handle splitting of the bufz across children
    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut BufZ) -> Self::O {
        loop {
            let prev_state = state.clone();
            let prev_bus = io.buf();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            state.0.strobe = self.strobe.sim(internal_inputs.strobe, &mut state.1, io);
            state.0.value = self.value.sim(internal_inputs.value, &mut state.2, io);
            state.0.buf_z = self.buf_z.sim(internal_inputs.buf_z, &mut state.3, io);
            note_push_path("side");
            state.0.side = self.side.sim(internal_inputs.side, &mut state.4, io);
            note_pop_path();
            note_push_path("latch");
            state.0.latch = self.latch.sim(internal_inputs.latch, &mut state.5, io);
            note_pop_path();
            if state == &prev_state && io.buf() == prev_bus {
                return outputs;
            }
        }
    }
}

#[kernel]
pub fn pushd(i: Clock, q: PushQ) -> (b8, PushD) {
    let mut d = PushD::default();
    d.buf_z.data = q.value;
    d.buf_z.enable = q.strobe & (q.side == Side::Left);
    d.strobe.clock = i;
    d.strobe.enable = true;
    d.side.clock = i;
    d.side.data = q.side;
    d.latch.clock = i;
    d.latch.data = q.latch;
    if q.strobe && (q.side == Side::Right) {
        d.latch.data = q.value;
    }
    if q.strobe {
        d.side.data = match q.side {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    };
    note("d", d);
    note("q", q);
    (q.latch, d)
}

#[test]
fn test_push_as_verilog() {
    let push = Push {
        strobe: Strobe::new(b32(5)),
        value: Constant::from(b8(5)),
        buf_z: ZDriver::default(),
        side: DFF::from(Side::Left),
        latch: DFF::from(b8(0)),
    };
    let top = push.as_hdl(crate::circuit::HDLKind::Verilog).unwrap();
    println!("{}", top);
}

#[test]
fn test_simulate_push() {
    let push = Push {
        strobe: Strobe::new(b32(10)),
        value: Constant::from(b8(5)),
        buf_z: ZDriver::default(),
        side: DFF::from(Side::Left),
        latch: DFF::from(b8(0)),
    };
    let mut state = push.init_state();
    note_init_db();
    note_time(0);
    for (ndx, input) in crate::clock::clock().take(1500).enumerate() {
        note_time(ndx as u64 * 100);
        note("clock", input);
        let mut buf = TristateBuf::default();
        buf.width = 8;
        loop {
            let p_state = state.clone();
            let p_buf = buf.clone();
            let mut io = BufZ::new(&mut buf, 0, 8);
            push.sim(input, &mut state, &mut io);
            if (state == p_state) && (p_buf == buf) {
                eprintln!("Stable[{ndx}]: {:?}", buf);
                break;
            }
        }
        note("bus", buf);
    }
    let db = note_take().unwrap();
    let push = std::fs::File::create("push.vcd").unwrap();
    db.dump_vcd(&[], push).unwrap();
}

#[derive(Clone)]
pub struct PushPair {
    left: Push,
    right: Push,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushPairQ {
    left: <Push as Circuit>::O,
    right: <Push as Circuit>::O,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushPairD {
    left: <Push as Circuit>::I,
    right: <Push as Circuit>::I,
}

impl Circuit for PushPair {
    type I = Clock;

    type O = (b8, b8);

    const NumZ: usize = 16;

    fn z_offsets() -> impl Iterator<Item = usize> {
        [0, 8].iter().copied()
    }

    type Q = PushPairQ;

    type D = PushPairD;

    type S = (Self::Q, <Push as Circuit>::S, <Push as Circuit>::S);

    type Update = push_pair;

    const UPDATE: crate::circuit::CircuitUpdateFn<Self> = push_pair;

    fn name(&self) -> &'static str {
        "PushPair"
    }

    fn init_state(&self) -> Self::S {
        (
            Default::default(),
            self.left.init_state(),
            self.right.init_state(),
        )
    }

    fn descriptor(&self) -> crate::circuit::CircuitDescriptor {
        let mut ret = root_descriptor(self);
        ret.children
            .insert("left".to_string(), self.left.descriptor());
        ret.children
            .insert("right".to_string(), self.right.descriptor());
        ret
    }

    fn as_hdl(&self, kind: crate::circuit::HDLKind) -> anyhow::Result<HDLDescriptor> {
        let mut ret = root_hdl(self, kind)?;
        ret.children
            .insert("left".to_string(), self.left.as_hdl(kind)?);
        ret.children
            .insert("right".to_string(), self.right.as_hdl(kind)?);
        Ok(ret)
    }

    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut BufZ) -> Self::O {
        loop {
            let prev_state = state.clone();
            let mut z_offsets = Self::z_offsets();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            note_push_path("left");
            state.0.left = self.left.sim(
                internal_inputs.left,
                &mut state.1,
                &mut io.shift(z_offsets.next().unwrap()),
            );
            note_pop_path();
            note_push_path("right");
            state.0.right = self.right.sim(
                internal_inputs.right,
                &mut state.2,
                &mut io.shift(z_offsets.next().unwrap()),
            );
            note_pop_path();
            if state == &prev_state {
                return outputs;
            }
        }
    }
}

#[kernel]
pub fn push_pair(i: Clock, q: PushPairQ) -> ((b8, b8), PushPairD) {
    let mut d = PushPairD::default();
    d.left = i;
    d.right = i;
    note("d", d);
    note("q", q);
    ((q.left, q.right), d)
}

#[test]
fn test_simulate_push_pair() {
    let push_left = Push {
        strobe: Strobe::new(b32(10)),
        value: Constant::from(b8(3)),
        buf_z: ZDriver::default(),
        side: DFF::from(Side::Left),
        latch: DFF::from(b8(0)),
    };
    let push_right = Push {
        strobe: Strobe::new(b32(10)),
        value: Constant::from(b8(5)),
        buf_z: ZDriver::default(),
        side: DFF::from(Side::Right),
        latch: DFF::from(b8(0)),
    };
    let push_pair = PushPair {
        left: push_left,
        right: push_right,
    };
    let mut state = push_pair.init_state();
    eprintln!("State: {:?}", state);
    note_init_db();
    note_time(0);
    for (ndx, input) in crate::clock::clock().take(1500).enumerate() {
        note_time(ndx as u64 * 100);
        note("clock", input);
        let mut buf = TristateBuf::default();
        buf.width = 16;
        loop {
            let p_state = state.clone();
            let p_buf = buf.clone();
            let mut io = BufZ::new(&mut buf, 0, 16);
            let output = push_pair.sim(input, &mut state, &mut io);
            fold_zbus::<8>(&mut buf);
            if (state == p_state) && (p_buf == buf) {
                note("output", output);
                break;
            }
        }
        note("bus", buf);
    }
    let db = note_take().unwrap();
    let push = std::fs::File::create("push_pair.vcd").unwrap();
    db.dump_vcd(&[], push).unwrap();
}

pub fn fold_zbus<const N: usize>(buf: &mut TristateBuf) {
    let left_value = buf.value >> N;
    let left_mask = buf.mask >> N;
    let right_value = buf.value & (Bits::<N>::MASK.0);
    let right_mask = buf.mask & (Bits::<N>::MASK.0);

    // Next we check that the two halves are not both enabled
    assert!(left_mask & right_mask == 0);
    // Next, we compute the combined value
    let value =
        ((left_value & left_mask) & !right_mask) | ((right_value & right_mask) & !left_mask);
    let total_mask = left_mask | right_mask;
    buf.value = value | value << N;
    buf.mask = total_mask | total_mask << N;
}

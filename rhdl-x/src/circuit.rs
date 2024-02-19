use anyhow::Result;
use rhdl_bits::Bits;
use rhdl_core::CircuitIO;
use rhdl_core::Digital;
use rhdl_core::Notable;
use std::hash::{Hash, Hasher};

pub type CircuitUpdateFn<C> =
    fn(<C as CircuitIO>::I, <C as Circuit>::Q) -> (<C as CircuitIO>::O, <C as Circuit>::D);

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct TristateBuf {
    pub width: usize,
    pub value: u128,
    pub mask: u128,
}

impl Notable for TristateBuf {
    fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
        writer.write_tristate(key, self.value, self.mask, self.width as u8);
    }
}

pub struct BufZ<'a> {
    bus: &'a mut TristateBuf,
    offset: usize,
    width: usize,
}

impl<'a> BufZ<'a> {
    pub fn shift(&mut self, offset: usize) -> BufZ {
        BufZ {
            bus: self.bus,
            offset: self.offset + offset,
            width: self.width,
        }
    }
    pub fn buf(&self) -> TristateBuf {
        *self.bus
    }
    pub fn new(bus: &'a mut TristateBuf, offset: usize, width: usize) -> Self {
        Self { bus, offset, width }
    }
    pub fn drive<const N: usize>(&mut self, value: Bits<N>) {
        self.bus.value &= !(value.0 << self.offset);
        self.bus.value |= value.0 << self.offset;
        self.bus.mask |= (Bits::<N>::MASK.0) << self.offset;
    }
    pub fn tri_state<const N: usize>(&mut self) {
        self.bus.mask &= !(Bits::<N>::MASK.0 << self.offset);
    }
    pub fn read<const N: usize>(&self) -> Bits<N> {
        rhdl_bits::bits::<N>((self.bus.value >> self.offset) & Bits::<N>::MASK.0)
    }
}

impl<'a> Notable for BufZ<'a> {
    fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
        writer.write_tristate(key, self.bus.value, self.bus.mask, self.width as u8);
    }
}

pub use rhdl_core::Tristate;

#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct BitZ<const N: usize> {
    pub value: Bits<N>,
    pub mask: Bits<N>,
}

impl<const N: usize> Notable for BitZ<N> {
    fn note(&self, key: impl rhdl_core::NoteKey, mut writer: impl rhdl_core::NoteWriter) {
        writer.write_tristate(key, self.value.0, self.mask.0, N as u8);
    }
}

impl<const N: usize> Tristate for BitZ<N> {
    const N: usize = N;
}

pub use rhdl_core::Circuit;
pub use rhdl_core::CircuitDescriptor;
pub use rhdl_core::HDLKind;

/*pub trait Circuit: 'static + Sized + Clone {
    // Input type - not auto derived
    type I: Digital;
    // Output type - not auto derived
    type O: Digital;

    // auto derived as the sum of NumZ of the children
    type Z: Tristate;

    type Update: DigitalFn;
    const UPDATE: CircuitUpdateFn<Self>;

    // Outputs of internal circuitry - auto derived
    type Q: Digital;
    // Inputs of internal circuitry - auto derived
    type D: Digital;

    // State for simulation - auto derived
    type S: Default + PartialEq + Clone;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S, io: &mut Self::Z) -> Self::O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }

    // auto derived
    fn name(&self) -> &'static str;

    // auto derived
    fn descriptor(&self) -> CircuitDescriptor;

    // auto derived
    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
}
*/
fn hash_id(fn_id: std::any::TypeId) -> u64 {
    // Hash the typeID into a 64 bit unsigned int
    let mut hasher = fnv::FnvHasher::default();
    fn_id.hash(&mut hasher);
    hasher.finish()
}

pub fn root_descriptor<C: Circuit>(circuit: &C) -> CircuitDescriptor {
    CircuitDescriptor {
        unique_name: format!(
            "{}_{:x}",
            circuit.name(),
            hash_id(std::any::TypeId::of::<C>())
        ),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        num_tristate: C::Z::N,
        tristate_offset_in_parent: 0,
        children: Default::default(),
    }
}

pub use rhdl_core::HDLDescriptor;

pub fn root_hdl<C: Circuit>(circuit: &C, kind: HDLKind) -> Result<HDLDescriptor> {
    match kind {
        HDLKind::Verilog => rhdl_core::root_verilog(circuit),
    }
}

pub trait UnsignedMethods<const N: usize> {
    fn set_bit(&mut self, index: u8, value: bool);
    fn get_bit(&self, index: u8) -> bool;
    fn any(&self) -> bool;
    fn all(&self) -> bool;
    fn xor(&self) -> bool;
    fn as_signed(&self) -> rhdl_bits::SignedBits<N>;
}

impl<const N: usize> UnsignedMethods<N> for rhdl_bits::Bits<N> {
    fn set_bit(&mut self, index: u8, value: bool) {
        *self = crate::set_bit::<N>(*self, index, value);
    }
    fn get_bit(&self, index: u8) -> bool {
        crate::get_bit::<N>(*self, index)
    }
    fn any(&self) -> bool {
        crate::any::<N>(*self)
    }
    fn all(&self) -> bool {
        crate::all::<N>(*self)
    }
    fn xor(&self) -> bool {
        crate::xor::<N>(*self)
    }
    fn as_signed(&self) -> rhdl_bits::SignedBits<N> {
        crate::as_signed::<N>(*self)
    }
}

pub trait SignedMethods<const N: usize> {
    fn sign_bit(&self) -> bool;
    fn as_unsigned(&self) -> rhdl_bits::Bits<N>;
}

impl<const N: usize> SignedMethods<N> for rhdl_bits::SignedBits<N> {
    fn sign_bit(&self) -> bool {
        crate::sign_bit::<N>(*self)
    }
    fn as_unsigned(&self) -> rhdl_bits::Bits<N> {
        crate::as_unsigned::<N>(*self)
    }
}

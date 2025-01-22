// We have several instances of implementation for a binary
// operator where the size semantics are preserved (i.e.,
// the result is the same size as the inputs).  To handle
// the cases we have a generic macro here, and then instantiate
// it as needed.
#[macro_export]
macro_rules! impl_binop {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding a u128 to a Bits<N>
        impl<N: BitWidth> $trait<u128> for Bits<N> {
            type Output = Bits<N>;
            fn $op(self, rhs: u128) -> Self::Output {
                assert!(rhs <= Self::MASK.val);
                bits_masked($wrap_op(self.val, rhs))
            }
        }
        // Next the case of adding a Bits<N> to a u128
        impl<N: BitWidth> $trait<Bits<N>> for u128 {
            type Output = Bits<N>;
            fn $op(self, rhs: Bits<N>) -> Self::Output {
                assert!(self <= Bits::<N>::MASK.val);
                bits_masked($wrap_op(self, rhs.val))
            }
        }
        // Adding two Bits<N> together
        impl<N: BitWidth> $trait for Bits<N> {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                bits_masked($wrap_op(self.val, rhs.val))
            }
        }
        // Adding a u128 to a DynBits
        impl $trait<u128> for DynBits {
            type Output = DynBits;
            fn $op(self, rhs: u128) -> Self::Output {
                assert!(rhs <= self.mask());
                DynBits {
                    val: $wrap_op(self.val, rhs),
                    bits: self.bits,
                }
                .wrapped()
            }
        }
        // Adding a DynBits to a u128
        impl $trait<DynBits> for u128 {
            type Output = DynBits;
            fn $op(self, rhs: DynBits) -> Self::Output {
                assert!(self <= rhs.mask());
                DynBits {
                    val: $wrap_op(self, rhs.val),
                    bits: rhs.bits,
                }
                .wrapped()
            }
        }
        // Adding two DynBits together
        impl $trait for DynBits {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                assert_eq!(self.bits, rhs.bits);
                DynBits {
                    val: $wrap_op(self.val, rhs.val),
                    bits: self.bits,
                }
                .wrapped()
            }
        }
        // Adding a DynBit to a Bits<N>
        impl<N: BitWidth> $trait<Bits<N>> for DynBits {
            type Output = Bits<N>;
            fn $op(self, rhs: Bits<N>) -> Self::Output {
                assert_eq!(self.bits, N::BITS);
                bits_masked($wrap_op(self.val, rhs.val))
            }
        }
        // Adding a Bits<N> to a DynBits
        impl<N: BitWidth> $trait<DynBits> for Bits<N> {
            type Output = Bits<N>;
            fn $op(self, rhs: DynBits) -> Self::Output {
                assert_eq!(rhs.bits, N::BITS);
                bits_masked($wrap_op(self.val, rhs.val))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_assign_op {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding a u128 to a Bits<N>
        impl<N: BitWidth> $trait<u128> for Bits<N> {
            fn $op(&mut self, rhs: u128) {
                assert!(rhs <= Self::MASK.val);
                *self = bits_masked($wrap_op(self.val, rhs));
            }
        }
        // Adding two Bits<N> together
        impl<N: BitWidth> $trait for Bits<N> {
            fn $op(&mut self, rhs: Self) {
                *self = bits_masked($wrap_op(self.val, rhs.val));
            }
        }
        // Adding a u128 to a DynBits
        impl $trait<u128> for DynBits {
            fn $op(&mut self, rhs: u128) {
                assert!(rhs <= self.mask());
                self.val = $wrap_op(self.val, rhs);
                *self = self.wrapped();
            }
        }
        // Adding two DynBits together
        impl $trait for DynBits {
            fn $op(&mut self, rhs: Self) {
                assert_eq!(self.bits, rhs.bits);
                self.val = $wrap_op(self.val, rhs.val);
                *self = self.wrapped();
            }
        }
        // Adding a DynBit to a Bits<N>
        impl<N: BitWidth> $trait<Bits<N>> for DynBits {
            fn $op(&mut self, rhs: Bits<N>) {
                assert_eq!(self.bits, N::BITS);
                self.val = $wrap_op(self.val, rhs.val);
                *self = self.wrapped();
            }
        }
        // Adding a Bits<N> to a DynBits
        impl<N: BitWidth> $trait<DynBits> for Bits<N> {
            fn $op(&mut self, rhs: DynBits) {
                assert_eq!(rhs.bits, N::BITS);
                *self = bits_masked($wrap_op(self.val, rhs.val));
            }
        }
    };
}

// Macro to generate impls for signed values
#[macro_export]
macro_rules! impl_signed_binop {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding an i128 to a SignedBits<N>
        impl<N: BitWidth> $trait<i128> for SignedBits<N> {
            type Output = SignedBits<N>;
            fn $op(self, rhs: i128) -> Self::Output {
                assert!(rhs <= Self::MAX);
                assert!(rhs >= Self::MIN);
                signed_wrapped($wrap_op(self.val, rhs))
            }
        }
        // Next the case of adding a SignedBits<N> to an i128
        impl<N: BitWidth> $trait<SignedBits<N>> for i128
        where
            N: BitWidth,
        {
            type Output = SignedBits<N>;
            fn $op(self, rhs: SignedBits<N>) -> Self::Output {
                assert!(self <= SignedBits::<N>::MAX);
                assert!(self >= SignedBits::<N>::MIN);
                signed_wrapped($wrap_op(self, rhs.val))
            }
        }
        // Adding two SignedBits<N> together
        impl<N: BitWidth> $trait for SignedBits<N> {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                signed_wrapped($wrap_op(self.val, rhs.val))
            }
        }
        // Adding a i128 to a SignedDynBits
        impl $trait<i128> for SignedDynBits {
            type Output = SignedDynBits;
            fn $op(self, rhs: i128) -> Self::Output {
                assert!(rhs <= self.max_value());
                assert!(rhs >= self.min_value());
                SignedDynBits {
                    val: $wrap_op(self.val, rhs),
                    bits: self.bits,
                }
                .wrapped()
            }
        }
        // Adding a SignedDynBits to a i128
        impl $trait<SignedDynBits> for i128 {
            type Output = SignedDynBits;
            fn $op(self, rhs: SignedDynBits) -> Self::Output {
                assert!(self <= rhs.max_value());
                assert!(self >= rhs.min_value());
                SignedDynBits {
                    val: $wrap_op(self, rhs.val),
                    bits: rhs.bits,
                }
                .wrapped()
            }
        }
        // Adding two SignedDynBits together
        impl $trait for SignedDynBits {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                assert_eq!(self.bits, rhs.bits);
                SignedDynBits {
                    val: $wrap_op(self.val, rhs.val),
                    bits: self.bits,
                }
                .wrapped()
            }
        }
        // Adding a SignedDynBit to a SignedBits<N>
        impl<N: BitWidth> $trait<SignedBits<N>> for SignedDynBits {
            type Output = SignedBits<N>;
            fn $op(self, rhs: SignedBits<N>) -> Self::Output {
                assert_eq!(self.bits, N::BITS);
                signed_wrapped($wrap_op(self.val, rhs.val))
            }
        }
        // Adding a SignedBits<N> to a SignedDynBits
        impl<N: BitWidth> $trait<SignedDynBits> for SignedBits<N> {
            type Output = SignedBits<N>;
            fn $op(self, rhs: SignedDynBits) -> Self::Output {
                assert_eq!(rhs.bits, N::BITS);
                signed_wrapped($wrap_op(self.val, rhs.val))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_assigned_signed_op {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding an i128 to a SignedBits<N>
        impl<N: BitWidth> $trait<i128> for SignedBits<N> {
            fn $op(&mut self, rhs: i128) {
                assert!(rhs <= Self::MAX);
                assert!(rhs >= Self::MIN);
                *self = signed_wrapped($wrap_op(self.val, rhs));
            }
        }
        // Adding two SignedBits<N> together
        impl<N: BitWidth> $trait for SignedBits<N> {
            fn $op(&mut self, rhs: Self) {
                *self = signed_wrapped($wrap_op(self.val, rhs.val));
            }
        }
        // Adding a i128 to a SignedDynBits
        impl $trait<i128> for SignedDynBits {
            fn $op(&mut self, rhs: i128) {
                assert!(rhs <= self.max_value());
                assert!(rhs >= self.min_value());
                self.val = $wrap_op(self.val, rhs);
                *self = self.wrapped();
            }
        }
        // Adding two SignedDynBits together
        impl $trait for SignedDynBits {
            fn $op(&mut self, rhs: Self) {
                assert_eq!(self.bits, rhs.bits);
                self.val = $wrap_op(self.val, rhs.val);
                *self = self.wrapped();
            }
        }
        // Adding a SignedDynBit to a SignedBits<N>
        impl<N: BitWidth> $trait<SignedBits<N>> for SignedDynBits {
            fn $op(&mut self, rhs: SignedBits<N>) {
                assert_eq!(self.bits, N::BITS);
                self.val = $wrap_op(self.val, rhs.val);
                *self = self.wrapped();
            }
        }
        // Adding a SignedBits<N> to a SignedDynBits
        impl<N: BitWidth> $trait<SignedDynBits> for SignedBits<N> {
            fn $op(&mut self, rhs: SignedDynBits) {
                assert_eq!(rhs.bits, N::BITS);
                *self = signed_wrapped($wrap_op(self.val, rhs.val));
            }
        }
    };
}

#[macro_export]
macro_rules! test_binop {
    ($op: tt, $wrap: path, $val1: expr, $val2: expr) => {
        {
            use $crate::rhdl_bits::alias::*;
            use $crate::rhdl_bits::bits;
            // We will generate some test data
            let x : b8 = bits($val1);
            let y : u128 = $val2;
            let z = x.dyn_bits();
            // Check all reasonable combinations.  For each combination,
            // the result should be the same as if we had added the two
            // values together and then masked the result.
            assert_eq!(x $op y, bits_masked($wrap(x.val, y)));
            assert_eq!(y $op x, bits_masked($wrap(y, x.val)));
            assert_eq!(x $op z, bits_masked($wrap(x.val, z.val)));
            assert_eq!(z $op x, bits_masked($wrap(z.val, x.val)));
            assert_eq!(z $op y, bits_masked::<U8>($wrap(z.val, y)).dyn_bits());
            assert_eq!(y $op z, bits_masked::<U8>($wrap(y, z.val)).dyn_bits());
            assert_eq!(z $op z, bits_masked::<U8>($wrap(z.val, z.val)).dyn_bits());
            assert_eq!(x $op x, bits_masked($wrap(x.val, x.val)));
            // Assert that all of the outputs are the same size ( 8 bits )
            assert_eq!((z $op y).bits(), 8);
            assert_eq!((y $op z).bits(), 8);
            assert_eq!((z $op z).bits(), 8);
            let _q: b8 = z.as_bits();
        }
    }
}

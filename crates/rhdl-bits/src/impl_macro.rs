// We have several instances of implementation for a binary
// operator where the size semantics are preserved (i.e.,
// the result is the same size as the inputs).  To handle
// the cases we have a generic macro here, and then instantiate
// it as needed.
#[doc(hidden)]
macro_rules! impl_binop {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding a u128 to a Bits<N>
        impl<const N: usize> $trait<u128> for Bits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = Bits<N>;
            fn $op(self, rhs: u128) -> Self::Output {
                assert!(rhs <= Self::MASK.raw());
                bits_masked($wrap_op(self.raw(), rhs))
            }
        }
        // Next the case of adding a Bits<N> to a u128
        impl<const N: usize> $trait<Bits<N>> for u128
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = Bits<N>;
            fn $op(self, rhs: Bits<N>) -> Self::Output {
                assert!(self <= Bits::<N>::MASK.raw());
                bits_masked($wrap_op(self, rhs.raw()))
            }
        }
        // Adding two Bits<N> together
        impl<const N: usize> $trait for Bits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                bits_masked($wrap_op(self.raw(), rhs.raw()))
            }
        }
        // Adding a u128 to a DynBits
        impl $trait<u128> for DynBits {
            type Output = DynBits;
            fn $op(self, rhs: u128) -> Self::Output {
                assert!(rhs <= self.mask());
                DynBits {
                    val: $wrap_op(self.raw(), rhs),
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
                    val: $wrap_op(self, rhs.raw()),
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
                    val: $wrap_op(self.raw(), rhs.raw()),
                    bits: self.bits,
                }
                .wrapped()
            }
        }
        // Adding a DynBit to a Bits<N>
        impl<const N: usize> $trait<Bits<N>> for DynBits
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = Bits<N>;
            fn $op(self, rhs: Bits<N>) -> Self::Output {
                assert_eq!(self.bits, { N });
                bits_masked($wrap_op(self.raw(), rhs.raw()))
            }
        }
        // Adding a Bits<N> to a DynBits
        impl<const N: usize> $trait<DynBits> for Bits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = Bits<N>;
            fn $op(self, rhs: DynBits) -> Self::Output {
                assert_eq!(rhs.bits, { N });
                bits_masked($wrap_op(self.raw(), rhs.raw()))
            }
        }
    };
}

macro_rules! impl_assign_op {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding a u128 to a Bits<N>
        impl<const N: usize> $trait<u128> for Bits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: u128) {
                assert!(rhs <= Self::MASK.raw());
                *self = bits_masked($wrap_op(self.raw(), rhs));
            }
        }
        // Adding two Bits<N> together
        impl<const N: usize> $trait for Bits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: Self) {
                *self = bits_masked($wrap_op(self.raw(), rhs.raw()));
            }
        }
        // Adding a u128 to a DynBits
        impl $trait<u128> for DynBits {
            fn $op(&mut self, rhs: u128) {
                assert!(rhs <= self.mask());
                self.val = $wrap_op(self.raw(), rhs);
                *self = self.wrapped();
            }
        }
        // Adding two DynBits together
        impl $trait for DynBits {
            fn $op(&mut self, rhs: Self) {
                assert_eq!(self.bits, rhs.bits);
                self.val = $wrap_op(self.raw(), rhs.raw());
                *self = self.wrapped();
            }
        }
        // Adding a DynBit to a Bits<N>
        impl<const N: usize> $trait<Bits<N>> for DynBits
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: Bits<N>) {
                assert_eq!(self.bits, { N });
                self.val = $wrap_op(self.raw(), rhs.raw());
                *self = self.wrapped();
            }
        }
        // Adding a Bits<N> to a DynBits
        impl<const N: usize> $trait<DynBits> for Bits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: DynBits) {
                assert_eq!(rhs.bits, { N });
                *self = bits_masked($wrap_op(self.raw(), rhs.raw()));
            }
        }
    };
}

// Macro to generate impls for signed values
macro_rules! impl_signed_binop {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding an i128 to a SignedBits<N>
        impl<const N: usize> $trait<i128> for SignedBits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = SignedBits<N>;
            fn $op(self, rhs: i128) -> Self::Output {
                assert!(rhs <= Self::MAX);
                assert!(rhs >= Self::MIN);
                signed_wrapped($wrap_op(self.raw(), rhs))
            }
        }
        // Next the case of adding a SignedBits<N> to an i128
        impl<const N: usize> $trait<SignedBits<N>> for i128
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = SignedBits<N>;
            fn $op(self, rhs: SignedBits<N>) -> Self::Output {
                assert!(self <= SignedBits::<N>::MAX);
                assert!(self >= SignedBits::<N>::MIN);
                signed_wrapped($wrap_op(self, rhs.raw()))
            }
        }
        // Adding two SignedBits<N> together
        impl<const N: usize> $trait for SignedBits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                signed_wrapped($wrap_op(self.raw(), rhs.raw()))
            }
        }
        // Adding a i128 to a SignedDynBits
        impl $trait<i128> for SignedDynBits {
            type Output = SignedDynBits;
            fn $op(self, rhs: i128) -> Self::Output {
                assert!(rhs <= self.max_value());
                assert!(rhs >= self.min_value());
                SignedDynBits {
                    val: $wrap_op(self.raw(), rhs),
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
                    val: $wrap_op(self, rhs.raw()),
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
                    val: $wrap_op(self.raw(), rhs.raw()),
                    bits: self.bits,
                }
                .wrapped()
            }
        }
        // Adding a SignedDynBit to a SignedBits<N>
        impl<const N: usize> $trait<SignedBits<N>> for SignedDynBits
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = SignedBits<N>;
            fn $op(self, rhs: SignedBits<N>) -> Self::Output {
                assert_eq!(self.bits, { N });
                signed_wrapped($wrap_op(self.raw(), rhs.raw()))
            }
        }
        // Adding a SignedBits<N> to a SignedDynBits
        impl<const N: usize> $trait<SignedDynBits> for SignedBits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            type Output = SignedBits<N>;
            fn $op(self, rhs: SignedDynBits) -> Self::Output {
                assert_eq!(rhs.bits, { N });
                signed_wrapped($wrap_op(self.raw(), rhs.raw()))
            }
        }
    };
}

macro_rules! impl_assigned_signed_op {
    ($trait: ident, $op: ident, $wrap_op: path) => {
        // First the case of adding an i128 to a SignedBits<N>
        impl<const N: usize> $trait<i128> for SignedBits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: i128) {
                assert!(rhs <= Self::MAX);
                assert!(rhs >= Self::MIN);
                *self = signed_wrapped($wrap_op(self.raw(), rhs));
            }
        }
        // Adding two SignedBits<N> together
        impl<const N: usize> $trait for SignedBits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: Self) {
                *self = signed_wrapped($wrap_op(self.raw(), rhs.raw()));
            }
        }
        // Adding a i128 to a SignedDynBits
        impl $trait<i128> for SignedDynBits {
            fn $op(&mut self, rhs: i128) {
                assert!(rhs <= self.max_value());
                assert!(rhs >= self.min_value());
                self.val = $wrap_op(self.raw(), rhs);
                *self = self.wrapped();
            }
        }
        // Adding two SignedDynBits together
        impl $trait for SignedDynBits {
            fn $op(&mut self, rhs: Self) {
                assert_eq!(self.bits, rhs.bits);
                self.val = $wrap_op(self.raw(), rhs.raw());
                *self = self.wrapped();
            }
        }
        // Adding a SignedDynBit to a SignedBits<N>
        impl<const N: usize> $trait<SignedBits<N>> for SignedDynBits
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: SignedBits<N>) {
                assert_eq!(self.bits, { N });
                self.val = $wrap_op(self.raw(), rhs.raw());
                *self = self.wrapped();
            }
        }
        // Adding a SignedBits<N> to a SignedDynBits
        impl<const N: usize> $trait<SignedDynBits> for SignedBits<N>
        where
            $crate::bitwidth::W<N>: BitWidth,
        {
            fn $op(&mut self, rhs: SignedDynBits) {
                assert_eq!(rhs.bits, { N });
                *self = signed_wrapped($wrap_op(self.raw(), rhs.raw()));
            }
        }
    };
}

#[cfg(test)]
macro_rules! test_binop {
    ($op: tt, $wrap: path, $val1: expr, $val2: expr) => {
        {
            use $crate::alias::*;
            use $crate::bits;
            // We will generate some test data
            let x : b8 = bits($val1);
            let y : u128 = $val2;
            let z = x.dyn_bits();
            // Check all reasonable combinations.  For each combination,
            // the result should be the same as if we had added the two
            // values together and then masked the result.
            assert_eq!(x $op y, bits_masked($wrap(x.raw(), y)));
            assert_eq!(y $op x, bits_masked($wrap(y, x.raw())));
            assert_eq!(x $op z, bits_masked($wrap(x.raw(), z.raw())));
            assert_eq!(z $op x, bits_masked($wrap(z.raw(), x.raw())));
            assert_eq!(z $op y, bits_masked::<8>($wrap(z.raw(), y)).dyn_bits());
            assert_eq!(y $op z, bits_masked::<8>($wrap(y, z.raw())).dyn_bits());
            assert_eq!(z $op z, bits_masked::<8>($wrap(z.raw(), z.raw())).dyn_bits());
            assert_eq!(x $op x, bits_masked($wrap(x.raw(), x.raw())));
            // Assert that all of the outputs are the same size ( 8 bits )
            assert_eq!((z $op y).bits(), 8);
            assert_eq!((y $op z).bits(), 8);
            assert_eq!((z $op z).bits(), 8);
            let _q: b8 = z.as_bits();
        }
    }
}

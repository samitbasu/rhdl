use rhdl::prelude::*;

#[test]
fn test_add_wraps() {
    // ANCHOR: add-wraps
    let a: b8 = 255.into();
    let b: b8 = a + 1; // No error.... but 
    assert_eq!(b, b8(0)); // b is zero!
    // ANCHOR_END: add-wraps
}

#[test]
fn test_sub_wraps() {
    // ANCHOR: sub-wraps
    let a: b8 = 0.into();
    let b: b8 = a - 1; // No error.... but 
    assert_eq!(b, b8(255)); // b is b8::MAX!
    // ANCHOR_END: sub-wraps
}

#[test]
fn test_negation_wraps() {
    // ANCHOR: neg-wraps
    let a: s8 = (-128).into(); // s8 can represent -128..127
    let b: s8 = -a; // No error.... but 
    assert_eq!(b, s8(-128)); // b is still -128!
    // ANCHOR_END: neg-wraps
}

#[test]
fn test_plus_equals() {
    // ANCHOR: plus-equals
    let mut a: b8 = 42.into();
    a += 1; // a is now 43
    assert_eq!(a, b8(43));
    // ANCHOR_END: plus-equals
}

#[test]
fn test_adecl() {
    // ANCHOR: alias-b13
    let a: b13;
    // ANCHOR_END: alias-b13
    let _ = a;
    // ANCHOR: alias-s12
    let c: s12;
    // ANCHOR_END: alias-s12
    let _ = c;
    // ANCHOR: explicit_versions
    let a: Bits<13>;
    let c: SignedBits<12>;
    // ANCHOR_END: explicit_versions
    let _ = a;
    let _ = c;
}

#[test]
fn test_where_for_fn() {
    // ANCHOR: where-for-fn
    fn my_func<const N: usize>(a: Bits<N>) -> Bits<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        let _c: SignedBits<N>;
        // Compute fancy stuff!
        a
    }
    // ANCHOR_END: where-for-fn
    let _ = my_func::<1>;
}

#[test]
fn test_get_all_bits() {
    // ANCHOR: get_all_bits
    let a = b8(200);
    let b = b8(100);
    let a = a.resize::<9>(); // zero extend to 9 bits
    let b = b.resize::<9>(); // ditto
    let c = a + b;
    let carry = c & (1 << 8) != 0;
    assert!(carry); // because 200 + 100 = 300, which needs 9 bits
    // ANCHOR_END: get_all_bits
}

#[test]
fn test_get_msb_function() {
    // ANCHOR: get_msb_function
    #[kernel] // <-- Synthesizable!
    fn get_msb<const N: usize>(a: Bits<N>) -> bool
    where
        rhdl::bits::W<N>: BitWidth,
    {
        (a & (1 << (N - 1))).any()
    }
    // ANCHOR_END: get_msb_function
    let _obj = compile_design::<get_msb<8>>(CompilationMode::Asynchronous).unwrap();
}

#[test]
fn test_comparison_ops() {
    // ANCHOR: comparison-ops
    let a: b8 = 42.into();
    let b: b8 = 65.into();
    let c = b > a;
    assert!(c); // c is a bool
    // ANCHOR_END: comparison-ops
}

#[test]
fn test_constructor_funcs() {
    // ANCHOR: constructor_into
    let a: b8 = 42.into();
    // ANCHOR_END: constructor_into
    let _ = a;
    // ANCHOR: constructor_b8
    let a = b8(42);
    // ANCHOR_END: constructor_b8
    let _ = a;
    // ANCHOR: constructor_bits
    let a: b8 = bits(42);
    // ANCHOR_END: constructor_bits
    let _ = a;
    // ANCHOR: constructor_bits_turbofish
    const N: usize = 8; // W<N>: BitWidth
    let a = bits::<N>(42);
    // ANCHOR_END: constructor_bits_turbofish
    let _ = a;
    // ANCHOR: constructor_s8
    let a: s8 = (-42).into(); // Works!  But is not synthesizable....
    // ANCHOR_END: constructor_s8
    let _ = a;
    // ANCHOR: constructor_s8_synth
    let a = s8(-42); // Works and is synthesizable
    let b: s8 = signed(-42); // Works and is also synthesizable
    let c = signed::<8>(-42); // Works and is synthesizable
    // ANCHOR_END: constructor_s8_synth
    let _ = b;
    let _ = c;
    let _ = a;
}

#[cfg(feature = "doc0")]
fn test_non_synthesizable() {
    // ANCHOR: constructor_s8_non_synth
    let a: s8 = -42.into(); // Doesn't work!
    // ANCHOR_END: constructor_s8_non_synth
}

#[test]
fn test_dyn_bits() {
    // ANCHOR: dyn-bits-ex
    let a: b8 = 12.into(); // 8 bits, compile time sized
    let b: b8 = 200.into(); // 8 bits, compile time sized
    let a = a.dyn_bits(); // 8 bits, run time sized
    let b = b.dyn_bits(); // 8 bits, run time sized
    let c = a.xadd(b); // 9 bits, run time sized
    let c: b9 = c.as_bits(); // 9 bits, compile time sized, run time checked
    // ANCHOR_END: dyn-bits-ex
    let _ = c;
}

#[test]
fn test_lerp() {
    // ANCHOR: lerp-ex
    fn lerp(a: b8, b: b8, x: b4) -> b8 {
        let a = a.dyn_bits();
        let b = b.dyn_bits();
        let prod_1 = a.xmul(x);
        let prod_2 = b.xmul(b5(16));
        let prod_3 = b.xmul(x);
        let sum = prod_1.xadd(prod_2.xadd(prod_3));
        let c = sum.xshr::<4>();
        c.resize::<8>().as_bits()
    }
    // ANCHOR_END: lerp-ex
    let c = lerp(b8(100), b8(200), b4(8));
    assert_eq!(c, b8(94));
}

// ANCHOR: late_checking_1
#[kernel]
fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Red>) -> Signal<b5, Red> {
    let a1 = a1.val().dyn_bits(); // 4 bits
    let a2 = a2.val().dyn_bits(); // 4 bits
    let c = a1.xadd(a2); // 5 bits
    let d = c.xadd(b1(1)); // 6 bits
    let e: b5 = d.as_bits(); // Uh oh!
    signal(e)
}
// ANCHOR_END: late_checking_1

#[cfg(feature = "doc2")]
// ANCHOR: late_checking_1_test
#[test]
fn test_run_do_stuff() {
    let y = do_stuff(signal(b4(3)), signal(b4(5))).val();
    assert_eq!(y, b5(9));
}
// ANCHOR_END: late_checking_1_test

#[cfg(feature = "doc3")]
// ANCHOR: compile_do_stuff
#[test]
fn test_compile_do_stuff() -> miette::Result<()> {
    compile_design::<do_stuff>(CompilationMode::Asynchronous)?;
    Ok(())
}
// ANCHOR_END: compile_do_stuff

#[test]
fn test_literal() {
    // ANCHOR: literal
    let a: b8 = 42.into();
    //         👇 - integer literal
    let b = a + 1;
    // ANCHOR_END: literal
    let _ = b;
}

#[test]
fn test_logical_ops() {
    // ANCHOR: logical
    let a: b8 = 0b0000_1011.into();
    let b: b8 = 0b1011_0011.into();
    let c = a | b; // 0b1011_1011
    let d = a & b; // 0b0000_0011
    let e = a ^ b; // 0b1011_1000
    let f = !a; // 0b1111_0100
    // ANCHOR_END: logical
    let _ = c;
    let _ = d;
    let _ = e;
    let _ = f;
}

#[test]
fn test_reduction_ops() {
    // ANCHOR: reduction
    let a: b8 = 0b0000_1011.into();
    let c: bool = a.any(); // equivalent to a != 0
    let d: bool = a.all(); // equivalent to (!a) == 0
    let e: bool = a.xor(); // true if an odd number of bits are 1, otherwise false
    // ANCHOR_END: reduction
    let _ = c;
    let _ = d;
    let _ = e;
}

#[test]
fn test_multiplication_op() {
    // ANCHOR: mul_unsigned
    let a: b8 = 12.into();
    let b: b8 = 3.into();
    let c: b8 = a * b; // = 36
    assert_eq!(c.raw(), 36);
    // ANCHOR_END: mul_unsigned
    // ANCHOR: mul_signed
    let a: s8 = 12.into();
    let b: s8 = (-3).into();
    let c: s8 = a * b; // = -36
    assert_eq!(c.raw(), -36);
    // ANCHOR_END: mul_signed
}

#[test]
fn left_shift() {
    // ANCHOR: left_shift
    let a: b8 = (0b1101_0011).into();
    let a = a << 1; // 0b1010_0110
    let a = a << 1; // 0b0100_1100
    assert_eq!(a, b8(0b0100_1100));
    // ANCHOR_END: left_shift
}

#[test]
fn signed_left_shift() {
    // ANCHOR: signed_left_shift
    let a: s8 = (0b0010_0101).into();
    let a = a << 1; // 0b0100_1010
    assert_eq!(a, s8(0b0100_1010));
    // ANCHOR_END: signed_left_shift
}

#[test]
fn right_shift() {
    // ANCHOR: right_shift
    let a: b8 = (0b1101_0011).into();
    let a = a >> 1; // 0b0110_1001
    let a = a >> 1; // 0b0011_0100
    assert_eq!(a, b8(0b0011_0100));
    // ANCHOR_END: right_shift
}

#[test]
fn signed_right_shift() {
    // ANCHOR: signed_right_shift
    let a: s8 = b8(0b1101_0011).as_signed();
    let a = a >> 1; // 0b1110_1001
    let a = a >> 1; // 0b1111_0100
    assert_eq!(a, b8(0b1111_0100).as_signed());
    let a: s8 = (0b0101_0011).into();
    let a = a >> 1; // 0b0010_1001
    let a = a >> 1; // 0b0001_0100
    assert_eq!(a, s8(0b0001_0100));
    // ANCHOR_END: signed_right_shift
}

#[test]
fn bit_bit_shift() {
    // ANCHOR: bit_bit_shift
    let a: b8 = (0b1101_0011).into();
    let b: b4 = 4.into();
    let c = a >> b; // 0b0000_1101
    assert_eq!(c, b8(0b0000_1101));
    // ANCHOR_END: bit_bit_shift
}

#[test]
fn test_basic_usage() {
    // ANCHOR: basic_usage
    let a = b4(6); // a is a 4-bit wide bit-vector
    let b = a; // It implements copy
    let c: b4 = b; // b4 is both the type and constructor name
    let d: Bits<4> = c; // Long form for writing a nibble.
    assert_eq!(d.raw(), 6); // raw() gets the integer value
    // ANCHOR_END: basic_usage
}

#[test]
fn test_basic_signed_usage() {
    // ANCHOR: basic_signed_usage
    let a = s4(-3); // a is a 4-bit wide signed bit-vector 
    let b = a; // It implements copy
    let c: s4 = b; // b4 is both the type and constructor name
    let d: SignedBits<4> = c; // Long form for writing a nibble.
    // ANCHOR_END: basic_signed_usage
    let _ = d;
}

#[cfg(feature = "doc4")]
#[test]
fn test_failed_bitcast() {
    // ANCHOR: failed_bitcast
    let a: b4 = b4(8);
    let b: b6 = a; // 👈 Illegal!
    // ANCHOR_END: failed_bitcast
}

#[cfg(feature = "doc5")]
#[test]
fn test_bool_not_b1() {
    // ANCHOR: bool_not_b1
    let a = b1(1);
    let b = b4(if a { 3 } else { 4 }); // 👈 Illegal! a is not a bool
    // ANCHOR_END: bool_not_b1
}

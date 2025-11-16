use std::env;
use std::io;
use std::io::Write;
use std::path::Path;

// For the binary ops, we need a description of the thing.  This is the universe of
// things we know how to operate on.
#[derive(Copy, Clone, PartialEq)]
enum TestThing {
    UnsignedLiteral(u128),
    SignedLiteral(i128),
    FixedBits(u8, u128),
    FixedSignedBits(u8, i128),
    DynBits(u8, u128),
    SignedDynBits(u8, i128),
}

impl std::fmt::Display for TestThing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TestThing::UnsignedLiteral(x) => format!("{x}_u128"),
                TestThing::SignedLiteral(x) => format!("({x} as i128)"),
                TestThing::FixedBits(n, x) => format!("b{n}({x})"),
                TestThing::FixedSignedBits(n, x) => format!("s{n}({x})"),
                TestThing::DynBits(n, x) => format!("b{n}({x}).dyn_bits()"),
                TestThing::SignedDynBits(n, x) => format!("s{n}({x}).dyn_bits()"),
            }
        )
    }
}

impl TestThing {
    fn as_dyn(val: Value) -> Option<Self> {
        match val {
            Value::Signed(x, bits) => bits.map(|bits| TestThing::SignedDynBits(bits, x)),
            Value::Unsigned(x, bits) => bits.map(|bits| TestThing::DynBits(bits, x)),
        }
    }
    fn as_static(val: Value) -> Option<Self> {
        match val {
            Value::Signed(x, bits) => bits.map(|bits| TestThing::FixedSignedBits(bits, x)),
            Value::Unsigned(x, bits) => bits.map(|bits| TestThing::FixedBits(bits, x)),
        }
    }
    fn is_dyn(self) -> bool {
        matches!(
            self,
            TestThing::DynBits(_, _) | TestThing::SignedDynBits(_, _)
        )
    }
    fn is_lit(self) -> bool {
        matches!(
            self,
            TestThing::UnsignedLiteral(_) | TestThing::SignedLiteral(_)
        )
    }
}

fn test_things() -> impl Iterator<Item = TestThing> {
    [
        TestThing::UnsignedLiteral(0),
        TestThing::UnsignedLiteral(1),
        TestThing::UnsignedLiteral(128),
        TestThing::UnsignedLiteral(32768),
        TestThing::SignedLiteral(0),
        TestThing::SignedLiteral(1),
        TestThing::SignedLiteral(128),
        TestThing::SignedLiteral(-1),
        TestThing::SignedLiteral(-128),
        TestThing::FixedBits(3, 0),
        TestThing::FixedBits(3, 7),
        TestThing::FixedBits(8, 0),
        TestThing::FixedBits(8, 1),
        TestThing::FixedBits(8, 128),
        TestThing::FixedBits(8, 255),
        TestThing::FixedBits(32, 0xDEAD_BEEF),
        TestThing::FixedSignedBits(8, 0),
        TestThing::FixedSignedBits(8, 1),
        TestThing::FixedSignedBits(8, 127),
        TestThing::FixedSignedBits(8, -128),
        TestThing::FixedSignedBits(8, -1),
        TestThing::DynBits(8, 0),
        TestThing::DynBits(8, 1),
        TestThing::DynBits(8, 255),
        TestThing::DynBits(32, 0xDEAD_BEEF),
        TestThing::DynBits(3, 0),
        TestThing::DynBits(3, 7),
        TestThing::SignedDynBits(8, 0),
        TestThing::SignedDynBits(8, 1),
        TestThing::SignedDynBits(8, 127),
        TestThing::SignedDynBits(8, -128),
        TestThing::SignedDynBits(8, -1),
    ]
    .into_iter()
}

#[derive(Copy, Clone, PartialEq)]
enum Value {
    Signed(i128, Option<u8>),
    Unsigned(u128, Option<u8>),
}

impl Value {
    fn bits(self) -> Option<u8> {
        match self {
            Value::Signed(_, bits) => bits,
            Value::Unsigned(_, bits) => bits,
        }
    }
    fn validate_length(self) -> Option<Self> {
        self.bits().filter(|&x| x > 0 && x <= 128).map(|_| self)
    }
    fn masked(self) -> Option<Self> {
        match self {
            Value::Unsigned(x, Some(bits)) => {
                let mask = u128::MAX >> (128 - bits);
                Some(Value::Unsigned(x & mask, Some(bits)))
            }
            Value::Signed(x, Some(bits)) => {
                // Mask it to the right size
                let mask = u128::MAX >> (128 - bits);
                let value = (x as u128) & mask;
                // Check the sign bit
                let sign_bit = 1_u128 << (bits - 1);
                let xval = if value & sign_bit != 0 {
                    (value | !mask) as i128
                } else {
                    value as i128
                };
                Some(Value::Signed(xval, Some(bits)))
            }
            _ => None,
        }
    }

    fn valid_with_bits(self, output_bits: u8) -> bool {
        match self {
            Value::Unsigned(x, None) => x <= (u128::MAX >> (128 - output_bits)),
            Value::Signed(x, None) => {
                x >= (i128::MIN >> (128 - output_bits)) && x <= (i128::MAX >> (128 - output_bits))
            }
            _ => true,
        }
    }
}

fn fixed_op(x: Option<u8>, y: Option<u8>) -> Option<u8> {
    match (x, y) {
        (Some(x), Some(y)) if x == y => Some(x),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        _ => None,
    }
}

fn fixed_dispatch(
    x: Value,
    y: Value,
    f: fn(u128, u128) -> u128,
    g: fn(i128, i128) -> i128,
) -> Option<Value> {
    if let Some(result) = match (x, y) {
        (Value::Unsigned(x, x_size), Value::Unsigned(y, y_size)) => {
            fixed_op(x_size, y_size).map(|bits| Value::Unsigned(f(x, y), Some(bits)))
        }
        (Value::Signed(x, x_size), Value::Signed(y, y_size)) => {
            fixed_op(x_size, y_size).map(|bits| Value::Signed(g(x, y), Some(bits)))
        }
        _ => None,
    } {
        let output_bits = result.bits().unwrap();
        if x.valid_with_bits(output_bits) && y.valid_with_bits(output_bits) {
            Some(result)
        } else {
            None
        }
    } else {
        None
    }
}

// We need a set of operations we can apply to Value.
impl std::ops::Add for Value {
    type Output = Option<Value>;
    fn add(self, rhs: Value) -> Self::Output {
        fixed_dispatch(self, rhs, u128::wrapping_add, i128::wrapping_add)
    }
}

impl std::ops::Sub for Value {
    type Output = Option<Value>;
    fn sub(self, rhs: Value) -> Self::Output {
        fixed_dispatch(self, rhs, u128::wrapping_sub, i128::wrapping_sub)
    }
}

impl std::ops::Mul for Value {
    type Output = Option<Value>;
    fn mul(self, rhs: Value) -> Self::Output {
        fixed_dispatch(self, rhs, u128::wrapping_mul, i128::wrapping_mul)
    }
}

fn fixed_bit_dispatch(x: Value, y: Value, f: fn(u128, u128) -> u128) -> Option<Value> {
    if let Some(result) = match (x, y) {
        (Value::Unsigned(x, x_size), Value::Unsigned(y, y_size)) => {
            fixed_op(x_size, y_size).map(|bits| Value::Unsigned(f(x, y), Some(bits)))
        }
        _ => None,
    } {
        let output_bits = result.bits().unwrap();
        if x.valid_with_bits(output_bits) && y.valid_with_bits(output_bits) {
            Some(result)
        } else {
            None
        }
    } else {
        None
    }
}

impl std::ops::BitAnd for Value {
    type Output = Option<Value>;
    fn bitand(self, rhs: Value) -> Self::Output {
        fixed_bit_dispatch(self, rhs, u128::bitand)
    }
}

impl std::ops::BitOr for Value {
    type Output = Option<Value>;
    fn bitor(self, rhs: Value) -> Self::Output {
        fixed_bit_dispatch(self, rhs, u128::bitor)
    }
}

impl std::ops::BitXor for Value {
    type Output = Option<Value>;
    fn bitxor(self, rhs: Value) -> Self::Output {
        fixed_bit_dispatch(self, rhs, u128::bitxor)
    }
}

fn shift_op_size(x: Option<u8>, _: Option<u8>) -> Option<u8> {
    x
}

fn shift_dispatch(
    x: Value,
    y: Value,
    f: fn(u128, u32, u8) -> Option<u128>,
    g: fn(i128, u32, u8) -> Option<i128>,
) -> Option<Value> {
    let output_size = shift_op_size(x.bits(), y.bits())?;
    match (x, y) {
        (Value::Unsigned(x, _), Value::Unsigned(y, _)) => {
            f(x, y as u32, output_size).map(|x| Value::Unsigned(x, Some(output_size)))
        }
        (Value::Signed(x, _), Value::Unsigned(y, _)) => {
            g(x, y as u32, output_size).map(|x| Value::Signed(x, Some(output_size)))
        }
        _ => None,
    }
}

fn wrap_shl(x: u128, y: u32, output_size: u8) -> Option<u128> {
    (y < output_size as u32).then(|| u128::wrapping_shl(x, y))
}

fn wrap_shl_signed(x: i128, y: u32, output_size: u8) -> Option<i128> {
    (y < output_size as u32).then(|| i128::wrapping_shl(x, y))
}

impl std::ops::Shl for Value {
    type Output = Option<Value>;
    fn shl(self, rhs: Value) -> Self::Output {
        shift_dispatch(self, rhs, wrap_shl, wrap_shl_signed)
    }
}

fn wrap_shr(x: u128, y: u32, output_size: u8) -> Option<u128> {
    (y < output_size as u32).then(|| u128::wrapping_shr(x, y))
}

fn wrap_shr_signed(x: i128, y: u32, output_size: u8) -> Option<i128> {
    (y < output_size as u32).then(|| i128::wrapping_shr(x, y))
}

impl std::ops::Shr for Value {
    type Output = Option<Value>;
    fn shr(self, rhs: Value) -> Self::Output {
        shift_dispatch(self, rhs, wrap_shr, wrap_shr_signed)
    }
}

impl TestThing {
    fn value(self) -> Value {
        match self {
            TestThing::UnsignedLiteral(x) => Value::Unsigned(x, None),
            TestThing::SignedLiteral(x) => Value::Signed(x, None),
            TestThing::FixedBits(n, x) => Value::Unsigned(x, Some(n)),
            TestThing::FixedSignedBits(n, x) => Value::Signed(x, Some(n)),
            TestThing::DynBits(n, x) => Value::Unsigned(x, Some(n)),
            TestThing::SignedDynBits(n, x) => Value::Signed(x, Some(n)),
        }
    }
}

// Next, we need a list of operations to test
#[derive(Copy, Clone, Debug, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    XAdd,
    XMul,
    XSub,
}

impl Op {
    fn is_shift(self) -> bool {
        matches!(self, Op::Shl | Op::Shr)
    }
    fn is_xop(self) -> bool {
        matches!(self, Op::XAdd | Op::XMul | Op::XSub)
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Add => "+",
                Op::Sub => "-",
                Op::Mul => "*",
                Op::And => "&",
                Op::Or => "|",
                Op::Xor => "^",
                Op::Shl => "<<",
                Op::Shr => ">>",
                Op::XAdd => ".xadd",
                Op::XMul => ".xmul",
                Op::XSub => ".xsub",
            }
        )
    }
}

fn xadd(x: Value, y: Value) -> Option<Value> {
    match (x, y) {
        (Value::Unsigned(x, Some(nx)), Value::Unsigned(y, Some(ny))) => {
            Some(Value::Unsigned(x.wrapping_add(y), Some(ny.max(nx) + 1)))
        }
        (Value::Signed(x, Some(nx)), Value::Signed(y, Some(ny))) => {
            Some(Value::Signed(x.wrapping_add(y), Some(ny.max(nx) + 1)))
        }
        _ => None,
    }
}

fn xmul(x: Value, y: Value) -> Option<Value> {
    match (x, y) {
        (Value::Unsigned(x, Some(nx)), Value::Unsigned(y, Some(ny))) => {
            Some(Value::Unsigned(x.wrapping_mul(y), Some(ny + nx)))
        }
        (Value::Signed(x, Some(nx)), Value::Signed(y, Some(ny))) => {
            Some(Value::Signed(x.wrapping_mul(y), Some(ny + nx)))
        }
        _ => None,
    }
}

fn xsub(x: Value, y: Value) -> Option<Value> {
    match (x, y) {
        (Value::Unsigned(x, Some(nx)), Value::Unsigned(y, Some(ny))) => {
            let x = x as i128;
            let y = y as i128;
            Some(Value::Signed(x.wrapping_sub(y), Some(ny.max(nx) + 1)))
        }
        (Value::Signed(x, Some(nx)), Value::Signed(y, Some(ny))) => {
            Some(Value::Signed(x.wrapping_sub(y), Some(ny.max(nx) + 1)))
        }
        _ => None,
    }
}

// Compute the stuff...
fn apply(op: Op, x: Value, y: Value) -> Option<Value> {
    match op {
        Op::Add => x + y,
        Op::Sub => x - y,
        Op::Mul => x * y,
        Op::And => x & y,
        Op::Or => x | y,
        Op::Xor => x ^ y,
        Op::Shl => x << y,
        Op::Shr => x >> y,
        Op::XAdd => xadd(x, y),
        Op::XMul => xmul(x, y),
        Op::XSub => xsub(x, y),
    }
    .and_then(|x| x.validate_length())
    .and_then(|x| x.masked())
}

// For a combination of a thing and an operation, we need a result.  The result will be a TestThing, but is
// optional.  Illegal combinations return None.
fn test_result_binop(thing1: TestThing, thing2: TestThing, op: Op) -> Option<TestThing> {
    // Compute the value for the output thing
    let output_value = apply(op, thing1.value(), thing2.value())?;
    if op.is_shift() {
        if thing1.is_dyn() {
            TestThing::as_dyn(output_value)
        } else {
            TestThing::as_static(output_value)
        }
    } else if (thing1.is_dyn() && (thing2.is_dyn() || thing2.is_lit()))
        || (thing2.is_dyn() && (thing1.is_dyn() || thing1.is_lit()))
        || op.is_xop()
    {
        TestThing::as_dyn(output_value)
    } else {
        TestThing::as_static(output_value)
    }
}

fn write_test_func<T: Write>(writer: &mut T, op: Op) {
    writeln!(writer, "#[test]").unwrap();
    writeln!(writer, "fn test_{op:?}() {{").unwrap();
    for thing1 in test_things() {
        for thing2 in test_things() {
            if let Some(result) = test_result_binop(thing1, thing2, op) {
                writeln!(
                    writer,
                    "{{
                        let t1 = {thing1};
                        let t2 = {thing2};
                        let t3 = t1 {op} (t2);
                        let t4 = {result};
                        assert_eq!(t3, t4);
                    }}"
                )
                .unwrap();
            }
        }
    }
    writeln!(writer, "}}").unwrap();
}

fn bit_tests() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("bit_tests.rs");
    let file = std::fs::File::create(&dest_path).unwrap();
    let mut io = io::BufWriter::new(file);
    writeln!(io, "use crate::alias::*;\n").unwrap();
    writeln!(io, "use crate::xsub::XSub;\n").unwrap();
    writeln!(io, "use crate::xmul::XMul;\n").unwrap();
    writeln!(io, "use crate::xadd::XAdd;\n").unwrap();
    write_test_func(&mut io, Op::Add);
    write_test_func(&mut io, Op::Sub);
    write_test_func(&mut io, Op::Mul);
    write_test_func(&mut io, Op::And);
    write_test_func(&mut io, Op::Or);
    write_test_func(&mut io, Op::Xor);
    write_test_func(&mut io, Op::Shl);
    write_test_func(&mut io, Op::Shr);
    write_test_func(&mut io, Op::XAdd);
    write_test_func(&mut io, Op::XMul);
    write_test_func(&mut io, Op::XSub);
}

fn main() {
    bit_tests();
    println!("cargo:rerun-if-changed=build.rs");
}

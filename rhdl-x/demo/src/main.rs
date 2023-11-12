use rhdl_bits::{alias::*, Bits};
use rhdl_bits::{bits, signed};
use rhdl_core::Digital;
use rhdl_macro::Digital;

#[derive(PartialEq, Copy, Clone)]
pub struct Foo {
    a: u8,
    b: u16,
    c: [u8; 3],
}

#[derive(PartialEq, Copy, Clone)]
pub enum NooState {
    Init,
    Run(u8, u8, u8),
    Walk { foo: u8 },
    Boom,
}

type nibble = Bits<4>;

#[derive(PartialEq, Copy, Clone)]
pub enum RedA {
    A,
    B(b4),
    C { x: b4, y: b6 },
}

#[derive(PartialEq, Copy, Clone)]
pub struct FooA {
    a: b8,
    b: s4,
    c: RedA,
}

#[derive(PartialEq, Copy, Clone)]
pub enum NooStateA {
    Init,
    Run(b4, b5),
    Walk { foo: b5 },
    Boom,
}

fn fifo(b: b8, a: b4) -> b8 {
    b
}

fn do_astuff_a(a: FooA, s: NooStateA) -> b7 {
    let z = (a.b, a.a);
    let q: b4 = 4.into() + 3.into();
    let foo = bits::<12>(6);
    let foo2 = (foo + foo);
    let c = a;
    let q = signed::<4>(2);
    let q = FooA {
        a: bits(1),
        b: q,
        c: RedA::A,
    };
    let c = RedA::A;
    let d = c;
    let z = fifo(bits(3), bits(5));
    let mut q = bits(1);
    let l = q.any();
    q.set_bit(3, true);
    let p = q.get_bit(2);
    let p = q.as_signed();
    if a.a > bits(0) {
        return bits(3);
    }
    let e = RedA::B(q);
    let x1 = bits(4);
    let y1 = bits(6);
    let mut ar = [bits(1), bits(1), bits(3)];
    ar[1] = bits(2);
    let z: [Bits<4>; 3] = ar;
    let q = ar[1];
    let f: [b4; 5] = [bits(1); 5];
    let h = f[2];
    let f = RedA::C { y: y1, x: x1 };
    let d = match s {
        NooStateA::Init => NooStateA::Run(bits(1), bits(2)),
        NooStateA::Run(x, y) => NooStateA::Walk { foo: y + 3 },
        NooStateA::Walk { foo: x } => {
            let q = bits(1) + x;
            NooStateA::Boom
        }
        NooStateA::Boom => NooStateA::Init,
        _ => NooStateA::Boom,
    };
    bits(42)
}

#[derive(PartialEq, Copy, Clone, Digital)]
struct FooN<T: Digital> {
    a: T,
    b: T,
}

fn do_m_stuff_nested<T: Digital, S: Digital>(x: FooN<T>, y: FooN<S>) -> bool {
    let c = x.a;
    let d = (x.a, y.b);
    let e = FooN { a: c, b: c };
    e == x
}

fn do_stuff(mut a: Foo, mut s: NooState) -> u8 {
    let k = {
        bits::<12>(4);
        bits::<12>(6)
    };
    let mut a: Foo = a;
    let mut s: NooState = s;
    let q: nibble = 3.into();
    let q = if a.a > 0 { bits::<12>(3) } else { bits(0) };
    let y = bits::<12>(72);
    let t2 = (y, y);
    let q: u8 = 4;
    let z = a.c;
    let w = (a, a);
    a.c[1] = q + 3;
    a.c = [0; 3];
    a.c = [1, 2, 3];
    let q = (1, (0, 5), 6);
    let (q0, (q1, q1b), q2): (u8, (u8, u8), u16) = q; // Tuple destructuring
    a.a = 2 + 3 + q1;
    let z;
    if 1 > 3 {
        z = bits::<4>(2);
    } else {
        z = bits::<4>(5);
    }
    a.b = {
        7 + 9;
        5 + !8
    };
    a.a = if 1 > 3 {
        7
    } else {
        {
            a.b = 1;
            a.b = 4;
        }
        9
    };
    let g = 1 > 2;
    let h = 3 != 4;
    let mut i = g && h;
    if z == bits::<4>(3) {
        i = false;
    }
    let c = match z {
        Bits(1) => 2,
        Bits(2) => 3,
        Bits(3) => {
            a.a = 4;
            4
        }
        _ => 6,
    };
    let d = match s {
        NooState::Init => {
            a.a = 1;
            NooState::Run(1, 2, 3)
        }
        NooState::Run(x, _, y) => {
            a.a = x + y;
            NooState::Walk { foo: 7 }
        }
        NooState::Walk { foo: x } => {
            a.a = x;
            NooState::Boom
        }
        NooState::Boom => {
            a.a = a.a + 3;
            NooState::Init
        }
        _ => {
            a.a = 2;
            NooState::Boom
        }
    };
    3
}

/*
*/

fn main() {
    println!("Hello, world!");
}

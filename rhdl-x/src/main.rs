use std::time::Instant;

use rhdl_bits::bits;
use rhdl_core::note_db::note_time;
use rhdl_core::{note, note_db::dump_vcd};
use rhdl_macro::kernel;
use rhdl_x::Foo;

use rhdl_bits::Bits;

#[kernel]
fn add_em2<const N: usize>(a: Bits<N>, b: Bits<N>) -> bool {
    note("a", a);
    rhdl_core::note("b", b);
    a == b
}

fn main() {
    let start = Instant::now();
    for i in 0..1_000_000 {
        note_time((i as u64) * 10);
        let foo_baz: Foo = Foo {
            field1: bits(i % 16),
            field2: bits(2),
            field3: (bits(i % 16), bits(i % 64)),
        };
        note("foo", foo_baz);
        let _res = add_em2(foo_baz.field1, foo_baz.field1);
    }
    eprintln!("{}ms", start.elapsed().as_millis());
    let mut s = vec![];
    let start = Instant::now();
    dump_vcd(&[], &mut s).unwrap();
    std::fs::write("test.vcd", s).unwrap();
    eprintln!("{}ms", start.elapsed().as_millis());
}

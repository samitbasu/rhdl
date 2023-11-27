use std::time::Instant;

use rhdl_bits::bits;
use rhdl_core::{note, note_db::dump};
use rhdl_x::Foo;

fn main() {
    let start = Instant::now();
    for i in 0..1_000_000 {
        let foo: Foo = Foo {
            field1: bits(i % 16),
            field2: bits(2),
            field3: (bits(i % 16), bits(i % 64)),
        };
        note("foo", foo);
    }
    eprintln!("{}ms", start.elapsed().as_millis());
    dump();
}

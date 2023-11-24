use std::time::Instant;

use rhdl_x::{dump, func_1, func_2, func_3, gump, gunk_1, gunk_2, gunk_3};

fn main() {
    func_1("foo");
    let now = Instant::now();
    for i in 0..10_000_000 {
        func_2(("foo", "[0]"), i % 2 == 0);
        func_3("foo", i);
    }
    eprintln!("{}ms", now.elapsed().as_millis());
    func_2(("foo", "[0]"), true);
    func_3("foo", 0);
    dump();
    gunk_1("foo");
    let now = Instant::now();
    for i in 0..10_000_000 {
        gunk_2(&format!("{}[0]", "foo"), i % 2 == 0);
        gunk_3("foo", i);
    }
    eprintln!("{}ms", now.elapsed().as_millis());
    gump();
}

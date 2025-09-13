use std::env;
use std::io;
use std::io::Write;
use std::path::Path;

fn write_typenum_impl_func(name: &str, op: &str, f: fn(usize, usize) -> Option<usize>) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(format!("typenum_{op}_impls.rs"));
    let file = std::fs::File::create(&dest_path).unwrap();
    let mut writer = io::BufWriter::new(file);
    for x in 1..=128 {
        for y in 1..=128 {
            if let Some(result) = f(x, y) {
                if result > 0 && result <= 128 {
                    writeln!(writer, "impl {name}<U{y}> for U{x} {{",).unwrap();
                    writeln!(writer, "    type Output = U{result};",).unwrap();
                    writeln!(writer, "   fn {op}(self, _: U{y}) -> Self::Output {{",).unwrap();
                    writeln!(writer, "        U{result}",).unwrap();
                    writeln!(writer, "    }}",).unwrap();
                    writeln!(writer, "}}").unwrap();
                }
            }
        }
    }
}

fn typenum_impls() {
    write_typenum_impl_func("std::ops::Add", "add", |x, y| (x + y <= 128).then(|| x + y));
    write_typenum_impl_func("std::ops::Sub", "sub", |x, y| (y < x).then(|| x - y));
    write_typenum_impl_func("Max", "max", |x, y| (x != y).then(|| x.max(y)));
    write_typenum_impl_func("Min", "min", |x, y| (x != y).then(|| x.min(y)));
}

fn main() {
    typenum_impls();
    println!("cargo:rerun-if-changed=build.rs");
}

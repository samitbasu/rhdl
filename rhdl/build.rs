use std::env;
use std::io;
use std::io::Write;
use std::path::Path;

// Exhaustively testing all the things takes _forever_.  So we dont.
fn test_seq() -> impl Iterator<Item = usize> {
    (0..=9).chain([
        10, 13, 15, 19, 20, 25, 22, 52, 40, 42, 44, 55, 60, 63, 67, 80, 83, 89, 91, 92, 99, 100,
        103, 110, 115, 120, 126, 128, 240, 449,
    ])
}

// Generate test code for a comparison function
fn write_comp_test_func<T: Write, F: Fn(usize, usize) -> Option<bool>>(
    writer: &mut T,
    op: F,
    name: &str,
) {
    writeln!(writer, "#[test]").unwrap();
    writeln!(writer, "fn test_{name}() {{").unwrap();
    for x in test_seq() {
        for y in test_seq() {
            if let Some(result) = (op)(x, y) {
                let trait_name = if result { "IsTrue" } else { "IsFalse" };
                writeln!(
                    writer,
                    "    assert_impl_all!({name}<U{x}, U{y}>: {trait_name});"
                )
                .unwrap();
            }
        }
    }
    writeln!(writer, "}}").unwrap();
}

// Generate test code for a binary function
// across the full range of supported ints
fn write_binop_test_func<T: Write, F: Fn(usize, usize) -> Option<usize>>(
    writer: &mut T,
    op: F,
    name: &str,
) {
    writeln!(writer, "#[test]").unwrap();
    writeln!(writer, "fn test_{name}() {{").unwrap();
    for x in test_seq() {
        for y in test_seq() {
            if let Some(result) = (op)(x, y) {
                writeln!(
                    writer,
                    "    assert_impl_all!(IsEqualTo<{name}<U{x}, U{y}>, U{result}>: IsTrue);"
                )
                .unwrap();
            }
        }
    }
    writeln!(writer, "}}").unwrap();
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tests.rs");
    //let dest_path = Path::new("jnk.rs");
    let file = std::fs::File::create(&dest_path).unwrap();
    let mut io = io::BufWriter::new(file);
    write_binop_test_func(&mut io, |x, y| Some(x + y), "Sum");
    write_binop_test_func(
        &mut io,
        |x, y| if x >= y { Some(x - y) } else { None },
        "Diff",
    );
    write_binop_test_func(&mut io, |x, y| Some(x.max(y)), "Maximum");
    write_binop_test_func(&mut io, |x, y| Some(x.min(y)), "Minimum");
    write_comp_test_func(&mut io, |x, y| Some(x < y), "IsLessThan");
    write_comp_test_func(&mut io, |x, y| Some(x > y), "IsGreaterThan");
    write_comp_test_func(&mut io, |x, y| Some(x <= y), "IsLessThanOrEqualTo");
    write_comp_test_func(&mut io, |x, y| Some(x >= y), "IsGreaterThanOrEqualTo");
    write_comp_test_func(&mut io, |x, y| Some(x == y), "IsEqualTo");
    write_comp_test_func(&mut io, |x, y| Some(x != y), "IsNotEqualTo");
    println!("cargo:rerun-if-changed=build.rs");
}

use itertools::Itertools;
// Take a function that implements
use rhdl_bits::{alias::*, bits};

use crate::{
    digital_fn::{inspect_digital, Describable, DigitalFn, KernelFnKind},
    kernel::ExternalKernelDef,
    Digital,
};
use anyhow::{bail, Result};

fn add(a: b4, b: b4) -> b4 {
    a + b
}

struct add {}

impl DigitalFn for add {
    fn kernel_fn() -> KernelFnKind {
        KernelFnKind::Extern(ExternalKernelDef {
            name: "add".to_string(),
            body: "function [3:0] add(input [3:0] a, input [3:0] b); add = a + b; endfunction"
                .to_string(),
        })
    }
}

fn make_test_module_2<F, T0, T1, T2>(
    f: F,
    desc: ExternalKernelDef,
    args: impl Iterator<Item = (T0, T1)>,
) -> String
where
    F: Fn(T0, T1) -> T2,
    T0: Digital,
    T1: Digital,
    T2: Digital,
{
    let ExternalKernelDef { name, body } = desc;
    let t0_bits = T0::static_kind().bits();
    let t1_bits = T1::static_kind().bits();
    let t2_bits = T2::static_kind().bits();
    let cases = args
        .map(|arg| {
            let t0 = arg.0.binary_string();
            let t1 = arg.1.binary_string();
            let t2 = f(arg.0, arg.1).binary_string();
            format!(
                "$display(\"0x%0h 0x%0h\", {t2_bits}'b{t2}, {}({t0_bits}'b{t0}, {t1_bits}'b{t1}));",
                name
            )
        })
        .collect::<Vec<_>>()
        .join("\n         ");
    format!(
        "
module testbench;
   {body}

   initial
      begin
         {cases}
         $finish;
       end
endmodule
    "
    )
}

#[test]
fn test_add() {
    let d = temp_dir::TempDir::new().unwrap();
    let nibbles_a = (0..=15).map(bits);
    let nibbles_b = nibbles_a.clone();
    let desc = match add::kernel_fn() {
        KernelFnKind::Extern(desc) => desc,
        _ => panic!("Expected an external kernel"),
    };
    let module = make_test_module_2(add, desc, nibbles_a.cartesian_product(nibbles_b));
    // Write the test bench to a file
    std::fs::write(d.path().join("testbench.v"), module).unwrap();
    // Compile the test bench
    let mut cmd = std::process::Command::new("iverilog");
    cmd.arg("-o")
        .arg(d.path().join("testbench"))
        .arg(d.path().join("testbench.v"));
    assert!(cmd.status().unwrap().success());
    let mut cmd = std::process::Command::new(d.path().join("testbench"));
    let output = cmd.output().unwrap();
    for case in String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.split(" ").collect::<Vec<_>>())
    {
        let expected = case[0];
        let actual = case[1];
        assert_eq!(expected, actual);
    }
}

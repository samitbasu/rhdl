use rhdl::prelude::*;

#[cfg(test)]
mod common;

use common::*;

#[test]
fn test_uninit_detection() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    pub struct Foo {
        a: b4,
        b: b4,
    }

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    pub enum Bar {
        A,
        B(b4),
        C,
        D,
    }

    #[kernel]
    fn foo(a: b4) -> b4 {
        let mut d = Bar::uninit();
        match d {
            Bar::B(b) => b,
            _ => a,
        }
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    let fg = build_rtl_flow_graph(&rtl);
    let file = std::fs::File::create("tests/uninit.dot").unwrap();
    write_dot(&fg, file).unwrap();
    eprintln!("{:?}", rtl);
    Ok(())
}

use rhdl::prelude::*;

use test_log::test;

#[derive(Circuit, Clone)]
struct XorGate;

impl CircuitDQ for XorGate {
    type D = ();
    type Q = ();
}

impl CircuitIO for XorGate {
    type I = Signal<(bool, bool), Red>;
    type O = Signal<bool, Red>;
    type Kernel = xor_gate;
}

#[kernel]
fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
    let (a, b) = i.val();
    let c = a ^ b;
    (signal(c), ())
}

#[test]
fn test_all_inputs() {
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let outputs = [false, true, true, false];
    inputs.iter().zip(outputs.iter()).for_each(|(inp, outp)| {
        let (y, _) = xor_gate(signal(*inp), ());
        assert_eq!(y.val(), *outp);
    });
}

#[test]
fn test_verilog_output() -> miette::Result<()> {
    let expect = expect_test::expect![[r#"
        module xor_gate(input wire [1:0] i, output wire [0:0] o);
           wire [0:0] od;
           assign o = od[0:0];
           assign od = kernel_xor_gate(i);
           function [0:0] kernel_xor_gate(input reg [1:0] arg_0);
                 reg [1:0] r0;
                 reg [0:0] r1;
                 reg [0:0] r2;
                 reg [0:0] r3;
                 begin
                    r0 = arg_0;
                    r1 = r0[0:0];
                    r2 = r0[1:1];
                    r3 = r1 ^ r2;
                    kernel_xor_gate = r3;
                 end
           endfunction
        endmodule
    "#]];
    let gate = XorGate;
    let hdl = gate.hdl("xor_gate")?;
    expect.assert_eq(&hdl.as_module().to_string());
    Ok(())
}

#[test]
fn test_simulation() {
    let gate = XorGate;
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let outputs = [false, true, true, false];
    let mut state = gate.init();
    for (inp, outp) in inputs.iter().zip(outputs.iter()) {
        let output = gate.sim(signal(*inp), &mut state);
        assert_eq!(output.val(), *outp);
    }
}

#[test]
fn test_svg() {
    let gate = XorGate;
    let inputs = [(false, false), (false, true), (true, false), (true, true)];
    let mut state = gate.init();
    let guard = trace_init_db();
    for (time, inp) in inputs.iter().enumerate() {
        trace_time((time * 100) as u64);
        let _output = gate.sim(signal(*inp), &mut state);
    }
    let svg = guard.take().dump_svg(0..=500, &Default::default());
    std::fs::write("xor.svg", svg.to_string()).unwrap();
}

#[test]
fn test_vcd() {
    let gate = XorGate;
    let inputs = [
        (false, false),
        (false, true),
        (true, false),
        (true, true),
        (false, false),
    ];
    let mut state = gate.init();
    let guard = trace_init_db();
    for (time, inp) in inputs.iter().enumerate() {
        trace_time((time * 100) as u64);
        let _output = gate.sim(signal(*inp), &mut state);
    }
    let vcd = std::fs::File::create("xor.vcd").unwrap();
    guard.take().dump_vcd(vcd, None).unwrap();
}

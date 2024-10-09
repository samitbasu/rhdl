use std::iter::once;

use rhdl_bits::alias::b1;

use crate::compiler::driver::{compile_design_stage1, compile_design_stage2};
use crate::error::RHDLError;
use crate::flow_graph::component::ComponentKind;
use crate::flow_graph::dot;
use crate::flow_graph::edge_kind::EdgeKind;
use crate::flow_graph::flow_graph_impl::FlowGraph;
use crate::flow_graph::hdl::generate_hdl;
use crate::flow_graph::passes::check_for_undriven::CheckForUndrivenPass;
use crate::flow_graph::passes::pass::Pass;
use crate::hdl::ast::{
    assert, assign, bit_string, component_instance, connection, declaration, delay, display,
    finish, id, initial, unsigned_width, Declaration, Function, HDLKind, Module, Statement,
};
use crate::hdl::builder::generate_verilog;
use crate::types::bit_string::BitString;
use crate::util::delim_list_optional_strings;
use crate::{build_rtl_flow_graph, DigitalFn};
use crate::{Timed, TypedBits};

pub trait TestArg {
    fn vec_tb(&self) -> Vec<TypedBits>;
}

impl<T0: Timed> TestArg for (T0,) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0,) = self;
        vec![t0.typed_bits()]
    }
}

impl<T0: Timed, T1: Timed> TestArg for (T0, T1) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0, t1) = self;
        vec![t0.typed_bits(), t1.typed_bits()]
    }
}

impl<T0: Timed, T1: Timed, T2: Timed> TestArg for (T0, T1, T2) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0, t1, t2) = self;
        vec![t0.typed_bits(), t1.typed_bits(), t2.typed_bits()]
    }
}

impl<T0: Timed, T1: Timed, T2: Timed, T3: Timed> TestArg for (T0, T1, T2, T3) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0, t1, t2, t3) = self;
        vec![
            t0.typed_bits(),
            t1.typed_bits(),
            t2.typed_bits(),
            t3.typed_bits(),
        ]
    }
}

impl<T0: Timed, T1: Timed, T2: Timed, T3: Timed, T4: Timed> TestArg for (T0, T1, T2, T3, T4) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0, t1, t2, t3, t4) = self;
        vec![
            t0.typed_bits(),
            t1.typed_bits(),
            t2.typed_bits(),
            t3.typed_bits(),
            t4.typed_bits(),
        ]
    }
}

pub trait Testable<Args, T1> {
    fn test_string(&self, name: &str, args: Args) -> String;
    fn apply(&self, args: Args) -> T1;
    fn declaration() -> Vec<Declaration>;
    fn test_case(&self, args: Args, ndx: usize) -> Vec<Statement>;
}

fn verilog_binary_string(x: impl Timed) -> Option<String> {
    let q = x.binary_string();
    if q.is_empty() {
        None
    } else {
        Some(format!("{x_bits}'b{q}", x_bits = q.len(), q = q))
    }
}

fn maybe_assign(target: &str, value: &BitString) -> Option<Statement> {
    (!value.is_empty()).then(|| assign(target, bit_string(value)))
}

impl<F, Q, T0> Testable<(T0,), Q> for F
where
    F: Fn(T0) -> Q,
    T0: Timed,
    Q: Timed,
{
    fn test_string(&self, name: &str, args: (T0,)) -> String {
        let (t0,) = args;
        let q = verilog_binary_string((*self)(t0)).unwrap();
        let t0 = verilog_binary_string(t0).unwrap_or_default();
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({t0}));\n")
    }
    fn apply(&self, args: (T0,)) -> Q {
        let (t0,) = args;
        (*self)(t0)
    }
    fn declaration() -> Vec<Declaration> {
        vec![
            declaration(HDLKind::Wire, "out", unsigned_width(Q::bits()), None),
            declaration(HDLKind::Reg, "arg_0", unsigned_width(T0::bits()), None),
        ]
    }
    fn test_case(&self, args: (T0,), ndx: usize) -> Vec<Statement> {
        let (t0,) = args;
        let b0: BitString = t0.typed_bits().into();
        let arg0 = maybe_assign("arg_0", &b0);
        let q = self.apply(args);
        let q: BitString = q.typed_bits().into();
        let d1 = delay(1);
        let assertion = assert(bit_string(&q), id("out"), ndx);
        // Create a vector of statements, with arg0, d1, and assertion, if arg0 is non-empty.  Otherwise, leave it out.
        arg0.into_iter()
            .chain(once(d1))
            .chain(once(assertion))
            .collect()
    }
}

impl<F, Q, T0, T1> Testable<(T0, T1), Q> for F
where
    F: Fn(T0, T1) -> Q,
    T0: Timed,
    T1: Timed,
    Q: Timed,
{
    fn test_string(&self, name: &str, args: (T0, T1)) -> String {
        let (t0, t1) = args;
        let q = verilog_binary_string((*self)(t0, t1)).unwrap();
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        let arg = delim_list_optional_strings(&[t0, t1], ",");
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({arg}));\n")
    }
    fn apply(&self, args: (T0, T1)) -> Q {
        let (t0, t1) = args;
        (*self)(t0, t1)
    }
    fn declaration() -> Vec<Declaration> {
        vec![
            declaration(HDLKind::Wire, "out", unsigned_width(Q::bits()), None),
            declaration(HDLKind::Reg, "arg_0", unsigned_width(T0::bits()), None),
            declaration(HDLKind::Reg, "arg_1", unsigned_width(T1::bits()), None),
        ]
    }
    fn test_case(&self, args: (T0, T1), ndx: usize) -> Vec<Statement> {
        let (t0, t1) = args;
        let b0: BitString = t0.typed_bits().into();
        let b1: BitString = t1.typed_bits().into();
        let arg0 = maybe_assign("arg_0", &b0);
        let arg1 = maybe_assign("arg_1", &b1);
        let q = self.apply(args);
        let q: BitString = q.typed_bits().into();
        let d1 = delay(1);
        let assertion = assert(bit_string(&q), id("out"), ndx);
        arg0.into_iter()
            .chain(arg1)
            .chain(once(d1))
            .chain(once(assertion))
            .collect()
    }
}

impl<F, Q, T0, T1, T2> Testable<(T0, T1, T2), Q> for F
where
    F: Fn(T0, T1, T2) -> Q,
    T0: Timed,
    T1: Timed,
    T2: Timed,
    Q: Timed,
{
    fn test_string(&self, name: &str, args: (T0, T1, T2)) -> String {
        let (t0, t1, t2) = args;
        let q = verilog_binary_string((*self)(t0, t1, t2)).unwrap();
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        let t2 = verilog_binary_string(t2);
        let arg = delim_list_optional_strings(&[t0, t1, t2], ",");
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({arg}));\n")
    }
    fn apply(&self, args: (T0, T1, T2)) -> Q {
        let (t0, t1, t2) = args;
        (*self)(t0, t1, t2)
    }
    fn declaration() -> Vec<Declaration> {
        vec![
            declaration(HDLKind::Wire, "out", unsigned_width(Q::bits()), None),
            declaration(HDLKind::Reg, "arg_0", unsigned_width(T0::bits()), None),
            declaration(HDLKind::Reg, "arg_1", unsigned_width(T1::bits()), None),
            declaration(HDLKind::Reg, "arg_2", unsigned_width(T2::bits()), None),
        ]
    }
    fn test_case(&self, args: (T0, T1, T2), ndx: usize) -> Vec<Statement> {
        let (t0, t1, t2) = args;
        let b0: BitString = t0.typed_bits().into();
        let b1: BitString = t1.typed_bits().into();
        let b2: BitString = t2.typed_bits().into();
        let arg0 = maybe_assign("arg_0", &b0);
        let arg1 = maybe_assign("arg_1", &b1);
        let arg2 = maybe_assign("arg_2", &b2);
        let q = self.apply(args);
        let q: BitString = q.typed_bits().into();
        let d1 = delay(1);
        let assertion = assert(bit_string(&q), id("out"), ndx);
        arg0.into_iter()
            .chain(arg1)
            .chain(arg2)
            .chain(once(d1))
            .chain(once(assertion))
            .collect()
    }
}

impl<F, Q, T0, T1, T2, T3> Testable<(T0, T1, T2, T3), Q> for F
where
    F: Fn(T0, T1, T2, T3) -> Q,
    T0: Timed,
    T1: Timed,
    T2: Timed,
    T3: Timed,
    Q: Timed,
{
    fn test_string(&self, name: &str, args: (T0, T1, T2, T3)) -> String {
        let (t0, t1, t2, t3) = args;
        let q = verilog_binary_string((*self)(t0, t1, t2, t3)).unwrap();
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        let t2 = verilog_binary_string(t2);
        let t3 = verilog_binary_string(t3);
        let arg = delim_list_optional_strings(&[t0, t1, t2, t3], ",");
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({arg}));\n")
    }
    fn apply(&self, args: (T0, T1, T2, T3)) -> Q {
        let (t0, t1, t2, t3) = args;
        (*self)(t0, t1, t2, t3)
    }
    fn declaration() -> Vec<Declaration> {
        vec![
            declaration(HDLKind::Wire, "out", unsigned_width(Q::bits()), None),
            declaration(HDLKind::Reg, "arg_0", unsigned_width(T0::bits()), None),
            declaration(HDLKind::Reg, "arg_1", unsigned_width(T1::bits()), None),
            declaration(HDLKind::Reg, "arg_2", unsigned_width(T2::bits()), None),
            declaration(HDLKind::Reg, "arg_3", unsigned_width(T3::bits()), None),
        ]
    }
    fn test_case(&self, args: (T0, T1, T2, T3), ndx: usize) -> Vec<Statement> {
        let (t0, t1, t2, t3) = args;
        let b0: BitString = t0.typed_bits().into();
        let b1: BitString = t1.typed_bits().into();
        let b2: BitString = t2.typed_bits().into();
        let b3: BitString = t3.typed_bits().into();
        let arg0 = maybe_assign("arg_0", &b0);
        let arg1 = maybe_assign("arg_1", &b1);
        let arg2 = maybe_assign("arg_2", &b2);
        let arg3 = maybe_assign("arg_3", &b3);
        let q = self.apply(args);
        let q: BitString = q.typed_bits().into();
        let d1 = delay(1);
        let assertion = assert(bit_string(&q), id("out"), ndx);
        arg0.into_iter()
            .chain(arg1)
            .chain(arg2)
            .chain(arg3)
            .chain(once(d1))
            .chain(once(assertion))
            .collect()
    }
}

impl<F, Q, T0, T1, T2, T3, T4> Testable<(T0, T1, T2, T3, T4), Q> for F
where
    F: Fn(T0, T1, T2, T3, T4) -> Q,
    T0: Timed,
    T1: Timed,
    T2: Timed,
    T3: Timed,
    T4: Timed,
    Q: Timed,
{
    fn test_string(&self, name: &str, args: (T0, T1, T2, T3, T4)) -> String {
        let (t0, t1, t2, t3, t4) = args;
        let q = verilog_binary_string((*self)(t0, t1, t2, t3, t4)).unwrap();
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        let t2 = verilog_binary_string(t2);
        let t3 = verilog_binary_string(t3);
        let t4 = verilog_binary_string(t4);
        let arg = delim_list_optional_strings(&[t0, t1, t2, t3, t4], ",");
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({arg}));\n")
    }
    fn apply(&self, args: (T0, T1, T2, T3, T4)) -> Q {
        let (t0, t1, t2, t3, t4) = args;
        (*self)(t0, t1, t2, t3, t4)
    }
    fn declaration() -> Vec<Declaration> {
        vec![
            declaration(HDLKind::Wire, "out", unsigned_width(Q::bits()), None),
            declaration(HDLKind::Reg, "arg_0", unsigned_width(T0::bits()), None),
            declaration(HDLKind::Reg, "arg_1", unsigned_width(T1::bits()), None),
            declaration(HDLKind::Reg, "arg_2", unsigned_width(T2::bits()), None),
            declaration(HDLKind::Reg, "arg_3", unsigned_width(T3::bits()), None),
            declaration(HDLKind::Reg, "arg_4", unsigned_width(T4::bits()), None),
        ]
    }
    fn test_case(&self, args: (T0, T1, T2, T3, T4), ndx: usize) -> Vec<Statement> {
        let (t0, t1, t2, t3, t4) = args;
        let b0: BitString = t0.typed_bits().into();
        let b1: BitString = t1.typed_bits().into();
        let b2: BitString = t2.typed_bits().into();
        let b3: BitString = t3.typed_bits().into();
        let b4: BitString = t4.typed_bits().into();
        let arg0 = maybe_assign("arg_0", &b0);
        let arg1 = maybe_assign("arg_1", &b1);
        let arg2 = maybe_assign("arg_2", &b2);
        let arg3 = maybe_assign("arg_3", &b3);
        let arg4 = maybe_assign("arg_4", &b4);
        let q = self.apply(args);
        let q: BitString = q.typed_bits().into();
        let d1 = delay(1);
        let assertion = assert(bit_string(&q), id("out"), ndx);
        arg0.into_iter()
            .chain(arg1)
            .chain(arg2)
            .chain(arg3)
            .chain(arg4)
            .chain(once(d1))
            .chain(once(assertion))
            .collect()
    }
}

fn test_module<F, Args, T0>(uut: F, desc: Function, vals: impl Iterator<Item = Args>) -> TestModule
where
    F: Testable<Args, T0>,
    T0: Timed,
{
    let name = &desc.name;
    let body = desc.as_verilog();
    let mut num_cases = 0;
    let cases = vals
        .inspect(|_| {
            num_cases += 1;
        })
        .map(|arg| uut.test_string(name, arg))
        .collect::<String>();
    TestModule {
        testbench: format!(
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
        ),
        num_cases,
    }
}

// In general, a flow graph cannot be reduced to a pure function, as it may contain internal
// state/etc.  However, in this context (for kernel testing), we can assume it is equivalent
// to a function, and generate the test vectors that way.  This is not equivalent to a full test
// module for the flow graph.  That has to go elsewhere.
fn test_module_for_flowgraph<F, Args, T0>(
    uut: F,
    desc: Module,
    vals: impl Iterator<Item = Args>,
) -> TestModule
where
    F: Testable<Args, T0>,
    T0: Timed,
{
    let name = &desc.name;
    let body = desc.as_verilog();
    let decls = F::declaration();
    let instance = component_instance(
        name,
        "t",
        decls
            .iter()
            .filter(|&decl| !decl.width.is_empty())
            .map(|decl| connection(&decl.name, id(&decl.name)))
            .collect(),
    );
    let mut statements = vec![];
    let mut num_cases = 0;
    statements.extend(
        vals.inspect(|_| {
            num_cases += 1;
        })
        .enumerate()
        .flat_map(|(ndx, arg)| uut.test_case(arg, ndx)),
    );
    statements.push(display("TESTBENCH OK", vec![]));
    statements.push(finish());
    let top = Module {
        name: "testbench".into(),
        ports: vec![],
        declarations: decls,
        statements: vec![instance, initial(statements)],
        functions: vec![],
    };
    TestModule {
        testbench: top.as_verilog() + &body,
        num_cases,
    }
}

pub struct TestModule {
    pub testbench: String,
    pub num_cases: usize,
}

impl TestModule {
    pub fn new<F, Args, T0>(uut: F, desc: Function, vals: impl Iterator<Item = Args>) -> TestModule
    where
        F: Testable<Args, T0>,
        T0: Timed,
    {
        test_module(uut, desc, vals)
    }
}

fn build_test_module_flowgraph(rtl: &crate::rtl::Object) -> FlowGraph {
    let internal_fg = build_rtl_flow_graph(rtl);
    // Create a new, top level FG with sources for the inputs and sinks for the
    // outputs.
    let mut fg = FlowGraph::default();
    let remap = fg.merge(&internal_fg);
    let timing_start = fg.new_component_with_optional_location(ComponentKind::TimingStart, 0, None);
    let timing_end = fg.new_component_with_optional_location(ComponentKind::TimingEnd, 0, None);
    // Create sources for all of the inputs of the internal flow graph
    internal_fg.inputs.iter().flatten().for_each(|input| {
        fg.edge(timing_start, remap[input], EdgeKind::Virtual);
    });
    internal_fg.output.iter().for_each(|output| {
        fg.edge(remap[output], timing_end, EdgeKind::Virtual);
    });
    // Create links from all of the internal sources to the timing start node
    let sources = fg
        .graph
        .node_indices()
        .filter(|node| matches!(fg.graph[*node].kind, ComponentKind::DFFOutput(_)))
        .collect::<Vec<_>>();
    let sinks = fg
        .graph
        .node_indices()
        .filter(|node| matches!(fg.graph[*node].kind, ComponentKind::DFFInput(_)))
        .collect::<Vec<_>>();
    sources.into_iter().for_each(|node| {
        fg.edge(timing_start, node, EdgeKind::Virtual);
    });
    sinks.into_iter().for_each(|node| {
        fg.edge(node, timing_end, EdgeKind::Virtual);
    });
    fg.inputs = vec![vec![timing_start]];
    fg.output = vec![timing_end];
    fg
}

pub fn test_kernel_vm_and_verilog<K, F, Args, T0>(
    uut: F,
    vals: impl Iterator<Item = Args> + Clone,
) -> Result<(), RHDLError>
where
    F: Testable<Args, T0>,
    T0: Timed,
    K: DigitalFn,
    Args: TestArg,
{
    let design = compile_design_stage1::<K>(crate::CompilationMode::Asynchronous)?;
    let rtl = compile_design_stage2(&design)?;
    let vm_inputs = vals.clone();
    let mut vm_test_count = 0;
    eprintln!("RHIF {:?}", design);
    for input in vm_inputs {
        let args_for_vm = input.vec_tb();
        let expected = uut.apply(input).typed_bits();
        let actual = crate::rhif::vm::execute(&design, args_for_vm)?;
        if expected.bits != actual.bits {
            return Err(RHDLError::VerilogVerificationErrorTyped { expected, actual });
        }
        vm_test_count += 1;
    }
    eprintln!("VM test passed {} cases OK", vm_test_count);
    let rtl_inputs = vals.clone();
    let mut rtl_test_count = 0;
    eprintln!("RTL {:?}", rtl);
    for input in rtl_inputs {
        let args_for_rtl: Vec<BitString> = input
            .vec_tb()
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();
        let expected: BitString = uut.apply(input).typed_bits().into();
        let actual = crate::rtl::vm::execute(&rtl, args_for_rtl)?;
        if expected.bits() != actual.bits() {
            return Err(RHDLError::VerilogVerificationErrorRTL { expected, actual });
        }
        rtl_test_count += 1;
    }
    eprintln!("RTL test passed {} cases OK", rtl_test_count);
    // let verilog = generate_verilog(&rtl)?;
    // let tm = test_module(uut, verilog, vals.clone());
    // tm.run_iverilog()?;
    let flow_graph = build_rtl_flow_graph(&rtl);
    let desc = generate_hdl("dut", &flow_graph)?;
    // Write the flow graph to a DOT file
    //    let flow_graph = CheckForUndrivenPass::run(flow_graph)?;
    // Use write_dot to write the flow graph out to a file
    //    std::fs::write("flow_graph.dot", flow_graph.dot()?)?;
    // eprintln!("{}", generate_hdl("top", &flow_graph)?.as_verilog());
    let tm = test_module_for_flowgraph(uut, desc, vals);
    tm.run_iverilog()?;
    Ok(())
}

impl std::fmt::Debug for TestModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.testbench.fmt(f)
    }
}

#[cfg(feature = "iverilog")]
impl TestModule {
    pub fn run_iverilog(&self) -> Result<(), RHDLError> {
        std::fs::write("testbench.v", &self.testbench)?;
        let d = tempfile::tempdir()?;
        // Write the test bench to a file
        let d_path = d.path();
        std::fs::write(d_path.join("testbench.v"), &self.testbench)?;
        // Compile the test bench
        let mut cmd = std::process::Command::new("iverilog");
        cmd.arg("-o")
            .arg(d_path.join("testbench"))
            .arg(d_path.join("testbench.v"));
        let status = cmd
            .status()
            .expect("Icarus Verilog should be installed and in your PATH.");
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to compile testbench with {}", status).into());
        }
        let mut cmd = std::process::Command::new("vvp");
        cmd.arg(d_path.join("testbench"));
        let output = cmd.output()?;
        match String::from_utf8_lossy(&output.stdout).lines().next() {
            Some(s) if s.trim() == "TESTBENCH OK" => Ok(()),
            Some(s) => Err(RHDLError::VerilogVerificationErrorString(s.into())),
            None => Err(RHDLError::VerilogVerificationErrorString(
                "No output".into(),
            )),
        }
    }
}

// This is split up so that in the future we can add additional
// test programs (verilator?) and still keep the back end in place.
#[cfg(feature = "iverilog")]
pub fn test_with_iverilog<F, Args, T0>(
    uut: F,
    desc: Function,
    vals: impl Iterator<Item = Args>,
) -> Result<(), RHDLError>
where
    F: Testable<Args, T0>,
    T0: Timed,
{
    test_module(uut, desc, vals).run_iverilog()
}

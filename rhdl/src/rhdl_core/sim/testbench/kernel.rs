use log::debug;
use std::iter::once;

use crate::rhdl_core::{
    build_rtl_flow_graph,
    compiler::driver::{compile_design_stage1, compile_design_stage2},
    flow_graph::{hdl::generate_hdl, optimization::optimize_flow_graph},
    hdl::{
        ast::{
            assert, assign, bit_string, component_instance, connection, continuous_assignment,
            declaration, delay, display, finish, function_call, id, initial, unsigned_width,
            Declaration, Function, HDLKind, Module, Statement,
        },
        builder::generate_verilog,
    },
    sim::test_module::TestModule,
    types::bit_string::BitString,
    Digital, DigitalFn, RHDLError, TypedBits,
};

pub trait TestArg {
    fn vec_tb(&self) -> Vec<TypedBits>;
}

impl<T0: Digital> TestArg for (T0,) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0,) = self;
        vec![t0.typed_bits()]
    }
}

impl<T0: Digital, T1: Digital> TestArg for (T0, T1) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0, t1) = self;
        vec![t0.typed_bits(), t1.typed_bits()]
    }
}

impl<T0: Digital, T1: Digital, T2: Digital> TestArg for (T0, T1, T2) {
    fn vec_tb(&self) -> Vec<TypedBits> {
        let (t0, t1, t2) = self;
        vec![t0.typed_bits(), t1.typed_bits(), t2.typed_bits()]
    }
}

impl<T0: Digital, T1: Digital, T2: Digital, T3: Digital> TestArg for (T0, T1, T2, T3) {
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

impl<T0: Digital, T1: Digital, T2: Digital, T3: Digital, T4: Digital> TestArg
    for (T0, T1, T2, T3, T4)
{
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
    fn apply(&self, args: Args) -> T1;
    fn declaration() -> Vec<Declaration>;
    fn test_case(&self, args: Args, ndx: usize) -> Vec<Statement>;
}

fn maybe_assign(target: &str, value: &BitString) -> Option<Statement> {
    (!value.is_empty()).then(|| assign(target, bit_string(value)))
}

impl<F, Q, T0> Testable<(T0,), Q> for F
where
    F: Fn(T0) -> Q,
    T0: Digital,
    Q: Digital,
{
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
        let d1 = delay(0);
        let assertion = assert(bit_string(&q), id("out"), &ndx.to_string());
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
    T0: Digital,
    T1: Digital,
    Q: Digital,
{
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
        let d1 = delay(0);
        let assertion = assert(bit_string(&q), id("out"), &ndx.to_string());
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
    T0: Digital,
    T1: Digital,
    T2: Digital,
    Q: Digital,
{
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
        let d1 = delay(0);
        let assertion = assert(bit_string(&q), id("out"), &ndx.to_string());
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
    T0: Digital,
    T1: Digital,
    T2: Digital,
    T3: Digital,
    Q: Digital,
{
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
        let d1 = delay(0);
        let assertion = assert(bit_string(&q), id("out"), &ndx.to_string());
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
    T0: Digital,
    T1: Digital,
    T2: Digital,
    T3: Digital,
    T4: Digital,
    Q: Digital,
{
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
        let d1 = delay(0);
        let assertion = assert(bit_string(&q), id("out"), &ndx.to_string());
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

fn test_module<F, Args, T0>(uut: &F, desc: Function, vals: impl Iterator<Item = Args>) -> TestModule
where
    F: Testable<Args, T0>,
    T0: Digital,
{
    let name = &desc.name;
    let decls = F::declaration();
    let arguments = decls
        .iter()
        .filter(|x| x.kind == HDLKind::Reg)
        .filter(|x| !x.width.is_empty())
        .map(|x| id(&x.name))
        .collect();
    let instance = continuous_assignment("out", function_call(name, arguments));
    let mut num_cases = 0;
    let mut cases = vals
        .inspect(|_| {
            num_cases += 1;
        })
        .enumerate()
        .flat_map(|(ndx, arg)| uut.test_case(arg, ndx))
        .collect::<Vec<_>>();
    cases.push(display("TESTBENCH OK", vec![]));
    cases.push(finish());
    let top = Module {
        name: "testbench".into(),
        description: format!("Autogenerated testbench for {}", name),
        declarations: decls,
        statements: vec![instance, initial(cases)],
        functions: vec![desc],
        ..Default::default()
    };
    top.into()
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
    T0: Digital,
{
    let name = &desc.name;
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
        description: format!("Autogenerated testbench for {}", name),
        declarations: decls,
        statements: vec![instance, initial(statements)],
        submodules: vec![desc],
        ..Default::default()
    };
    top.into()
}

fn test_kernel_vm_and_verilog_with_mode<K, F, Args, T0>(
    uut: F,
    vals: impl Iterator<Item = Args> + Clone,
    mode: crate::rhdl_core::CompilationMode,
) -> Result<(), RHDLError>
where
    F: Testable<Args, T0>,
    T0: Digital,
    K: DigitalFn,
    Args: TestArg,
{
    let design = compile_design_stage1::<K>(mode)?;
    let rtl = compile_design_stage2(&design)?;
    let vm_inputs = vals.clone();
    debug!("Testing kernel function");
    debug!("----- RHIF -----");
    debug!("{:?}", design);
    debug!("----- RTL ------");
    debug!("{:?}", rtl);
    debug!("Running RHIF VM check");
    for input in vm_inputs {
        let args_for_vm = input.vec_tb();
        let expected = uut.apply(input).typed_bits();
        let actual = crate::rhdl_core::rhif::vm::execute(&design, args_for_vm)?;
        if expected.bits != actual.bits {
            return Err(RHDLError::VerilogVerificationErrorTyped { expected, actual });
        }
    }
    let rtl_inputs = vals.clone();
    debug!("Running RTL VM check");
    for input in rtl_inputs {
        let args_for_rtl: Vec<BitString> = input
            .vec_tb()
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();
        let expected: BitString = uut.apply(input).typed_bits().into();
        let actual = crate::rhdl_core::rtl::vm::execute(&rtl, args_for_rtl)?;
        if expected.bits() != actual.bits() {
            return Err(RHDLError::VerilogVerificationErrorRTL { expected, actual });
        }
    }
    debug!("Generating Verilog to run external checks");
    let hdl = generate_verilog(&rtl)?;
    let tm = test_module(&uut, hdl, vals.clone());
    tm.run_iverilog()?;
    let flow_graph = build_rtl_flow_graph(&rtl);
    let flow_graph = optimize_flow_graph(flow_graph)?;
    let desc = generate_hdl("dut", &flow_graph)?;
    let tm = test_module_for_flowgraph(uut, desc, vals);
    tm.run_iverilog()?;
    Ok(())
}

pub fn test_kernel_vm_and_verilog<K, F, Args, T0>(
    uut: F,
    vals: impl Iterator<Item = Args> + Clone,
) -> Result<(), RHDLError>
where
    F: Testable<Args, T0>,
    T0: Digital,
    K: DigitalFn,
    Args: TestArg,
{
    test_kernel_vm_and_verilog_with_mode::<K, F, Args, T0>(
        uut,
        vals,
        crate::rhdl_core::CompilationMode::Asynchronous,
    )
}

pub fn test_kernel_vm_and_verilog_synchronous<K, F, Args, T0>(
    uut: F,
    vals: impl Iterator<Item = Args> + Clone,
) -> Result<(), RHDLError>
where
    F: Testable<Args, T0>,
    T0: Digital,
    K: DigitalFn,
    Args: TestArg,
{
    test_kernel_vm_and_verilog_with_mode::<K, F, Args, T0>(
        uut,
        vals,
        crate::rhdl_core::CompilationMode::Synchronous,
    )
}

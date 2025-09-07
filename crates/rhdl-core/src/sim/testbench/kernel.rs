use log::debug;
use rhdl_vlog::Pretty;

use crate::{
    Digital, DigitalFn, RHDLError, TypedBits,
    compiler::{
        driver::{compile_design_stage1, compile_design_stage2},
        optimize_ntl,
    },
    ntl::from_rtl::build_ntl_from_rtl,
    sim::test_module::TestModule,
    types::bit_string::BitString,
};

use quote::{format_ident, quote};
use rhdl_vlog as vlog;
use syn::parse_quote;

fn maybe_assign(target: &str, value: &TypedBits) -> Option<vlog::Stmt> {
    (!value.is_empty()).then(|| {
        let target = format_ident!("{target}");
        let value: vlog::LitVerilog = value.into();
        parse_quote! { #target = #value }
    })
}

fn assert_stmt(left: TypedBits, right: &str, msg: &str) -> vlog::Stmt {
    let left: vlog::LitVerilog = (&left).into();
    let right = format_ident!("{right}");
    let message = format!("ASSERTION FAILED 0x%0h !== 0x%0h CASE {msg}");
    parse_quote! {
        if ((#left) !== (#right)) begin
            $display(#message, #left, #right);
            $finish;
        end
    }
}

fn build_test_case(
    args: impl IntoIterator<Item = TypedBits>,
    q: TypedBits,
    ndx: usize,
) -> vlog::stmt::StmtList {
    let arguments = args
        .into_iter()
        .enumerate()
        .flat_map(|(ndx, arg)| maybe_assign(&format!("arg_{ndx}"), &arg));
    let delay = vlog::delay_stmt(0);
    let assertion = assert_stmt(q, "out", &ndx.to_string());
    parse_quote! {
        #(#arguments)*
        #delay ;
        #assertion;
    }
}

fn decl_list(q_len: usize, arg_sizes: &[usize]) -> Vec<vlog::Declaration> {
    std::iter::once(vlog::wire_decl("out", vlog::unsigned_width(q_len)))
        .chain(
            arg_sizes.iter().enumerate().map(|(ndx, &size)| {
                vlog::reg_decl(&format!("arg_{ndx}"), vlog::unsigned_width(size))
            }),
        )
        .collect()
}

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
    fn declaration() -> Vec<vlog::Declaration>;
    fn test_case(&self, args: Args, ndx: usize) -> vlog::stmt::StmtList;
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
    fn declaration() -> Vec<vlog::Declaration> {
        decl_list(Q::bits(), &[T0::bits()])
    }
    fn test_case(&self, args: (T0,), ndx: usize) -> vlog::stmt::StmtList {
        let (t0,) = args;
        let q = self.apply(args);
        build_test_case([t0.typed_bits()], q.typed_bits(), ndx)
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
    fn declaration() -> Vec<vlog::Declaration> {
        decl_list(Q::bits(), &[T0::bits(), T1::bits()])
    }
    fn test_case(&self, args: (T0, T1), ndx: usize) -> vlog::stmt::StmtList {
        let (t0, t1) = args;
        let q = self.apply(args);
        build_test_case([t0.typed_bits(), t1.typed_bits()], q.typed_bits(), ndx)
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
    fn declaration() -> Vec<vlog::Declaration> {
        decl_list(Q::bits(), &[T0::bits(), T1::bits(), T2::bits()])
    }
    fn test_case(&self, args: (T0, T1, T2), ndx: usize) -> vlog::stmt::StmtList {
        let (t0, t1, t2) = args;
        let q = self.apply(args);
        build_test_case(
            [t0.typed_bits(), t1.typed_bits(), t2.typed_bits()],
            q.typed_bits(),
            ndx,
        )
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
    fn declaration() -> Vec<vlog::Declaration> {
        decl_list(Q::bits(), &[T0::bits(), T1::bits(), T2::bits(), T3::bits()])
    }
    fn test_case(&self, args: (T0, T1, T2, T3), ndx: usize) -> vlog::stmt::StmtList {
        let (t0, t1, t2, t3) = args;
        let q = self.apply(args);
        build_test_case(
            [
                t0.typed_bits(),
                t1.typed_bits(),
                t2.typed_bits(),
                t3.typed_bits(),
            ],
            q.typed_bits(),
            ndx,
        )
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
    fn declaration() -> Vec<vlog::Declaration> {
        decl_list(
            Q::bits(),
            &[T0::bits(), T1::bits(), T2::bits(), T3::bits(), T4::bits()],
        )
    }
    fn test_case(&self, args: (T0, T1, T2, T3, T4), ndx: usize) -> vlog::stmt::StmtList {
        let (t0, t1, t2, t3, t4) = args;
        let q = self.apply(args);
        build_test_case(
            [
                t0.typed_bits(),
                t1.typed_bits(),
                t2.typed_bits(),
                t3.typed_bits(),
                t4.typed_bits(),
            ],
            q.typed_bits(),
            ndx,
        )
    }
}

fn test_module<F, Args, T0>(
    uut: &F,
    desc: vlog::FunctionDef,
    vals: impl Iterator<Item = Args>,
) -> TestModule
where
    F: Testable<Args, T0>,
    T0: Digital,
{
    let name = format_ident!("{}", desc.name);
    let decls = F::declaration();
    let arguments = decls
        .iter()
        .filter(|x| x.kind.is_reg())
        .map(|x| format_ident!("{}", &x.name));
    let cases = vals
        .enumerate()
        .flat_map(|(ndx, arg)| uut.test_case(arg, ndx).0);
    let module: vlog::ModuleList = parse_quote! {
        module testbench;
            #(#decls;)*
            assign out = #name(#(#arguments,)*);
            initial begin
                #(#cases;)*
                $display("TESTBENCH OK");
                $finish;
            end
            #desc
        endmodule
    };
    log::info!("Generated test module:\n{}", module.pretty());
    module.into()
}

// In general, a netlist cannot be reduced to a pure function, as it may contain internal
// state/etc.  However, in this context (for kernel testing), we can assume it is equivalent
// to a function, and generate the test vectors that way.  This is not equivalent to a full test
// module for the netlist.  That has to go elsewhere.
fn test_module_for_netlist<F, Args, T0>(
    uut: F,
    desc: vlog::ModuleList,
    vals: impl Iterator<Item = Args>,
) -> TestModule
where
    F: Testable<Args, T0>,
    T0: Digital,
{
    let name = format_ident!("dut");
    let decls = F::declaration();
    let connections = decls.iter().map(|decl| {
        let name = format_ident!("{}", decl.name);
        quote! {.#name(#name)}
    });
    let cases = vals
        .enumerate()
        .flat_map(|(ndx, arg)| uut.test_case(arg, ndx).0);
    let module: vlog::ModuleList = parse_quote! {
        module testbench;
            #(#decls)*
            #name t(#(#connections),*);
            initial begin
                #(#cases)*
                $display("TESTBENCH OK");
                $finish;
            end
        endmodule
        #desc
    };
    module.into()
}

fn test_kernel_vm_and_verilog_with_mode<K, F, Args, T0>(
    uut: F,
    vals: impl Iterator<Item = Args> + Clone,
    mode: crate::CompilationMode,
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
        let actual = crate::rhif::vm::execute(&design, args_for_vm)?;
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
        let actual = crate::rtl::vm::execute(&rtl, args_for_rtl)?;
        if expected.bits() != actual.bits() {
            return Err(RHDLError::VerilogVerificationErrorRTL { expected, actual });
        }
    }
    debug!("Generating Verilog to run external checks");
    let vlog = rtl.as_vlog()?;
    debug!("{}", vlog.pretty());
    let tm = test_module(&uut, vlog, vals.clone());
    tm.run_iverilog()?;
    debug!("Generating netlist from rtl");
    let ntl = build_ntl_from_rtl(&rtl);
    let ntl = optimize_ntl(ntl)?;
    debug!("{rtl:?}");
    debug!("{ntl:?}");
    let desc = ntl.as_vlog("dut")?;
    let tm = test_module_for_netlist(uut, desc, vals);
    debug!("Running netlist test");
    debug!("{}", tm);
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
        crate::CompilationMode::Asynchronous,
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
        crate::CompilationMode::Synchronous,
    )
}

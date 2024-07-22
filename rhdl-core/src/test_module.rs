use crate::compiler::codegen::verilog::generate_verilog;
use crate::error::RHDLError;
use crate::rhif::vm::execute_function;
use crate::{compile_design, DigitalFn};
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
}

fn verilog_binary_string(x: impl Timed) -> String {
    let q = x.binary_string();
    if q.is_empty() {
        "0".to_string()
    } else {
        format!("{x_bits}'b{q}", x_bits = q.len(), q = q)
    }
}

impl<F, Q, T0> Testable<(T0,), Q> for F
where
    F: Fn(T0) -> Q,
    T0: Timed,
    Q: Timed,
{
    fn test_string(&self, name: &str, args: (T0,)) -> String {
        let (t0,) = args;
        let q = verilog_binary_string((*self)(t0));
        let t0 = verilog_binary_string(t0);
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({t0}));\n")
    }
    fn apply(&self, args: (T0,)) -> Q {
        let (t0,) = args;
        (*self)(t0)
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
        let q = verilog_binary_string((*self)(t0, t1));
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({t0},{t1}));\n")
    }
    fn apply(&self, args: (T0, T1)) -> Q {
        let (t0, t1) = args;
        (*self)(t0, t1)
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
        let q = verilog_binary_string((*self)(t0, t1, t2));
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        let t2 = verilog_binary_string(t2);
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({t0},{t1},{t2}));\n")
    }
    fn apply(&self, args: (T0, T1, T2)) -> Q {
        let (t0, t1, t2) = args;
        (*self)(t0, t1, t2)
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
        let q = verilog_binary_string((*self)(t0, t1, t2, t3));
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        let t2 = verilog_binary_string(t2);
        let t3 = verilog_binary_string(t3);
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({t0},{t1},{t2},{t3}));\n")
    }
    fn apply(&self, args: (T0, T1, T2, T3)) -> Q {
        let (t0, t1, t2, t3) = args;
        (*self)(t0, t1, t2, t3)
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
        let q = verilog_binary_string((*self)(t0, t1, t2, t3, t4));
        let t0 = verilog_binary_string(t0);
        let t1 = verilog_binary_string(t1);
        let t2 = verilog_binary_string(t2);
        let t3 = verilog_binary_string(t3);
        let t4 = verilog_binary_string(t4);
        format!("$display(\"0x%0h 0x%0h\", {q}, {name}({t0},{t1},{t2},{t3},{t4}));\n")
    }
    fn apply(&self, args: (T0, T1, T2, T3, T4)) -> Q {
        let (t0, t1, t2, t3, t4) = args;
        (*self)(t0, t1, t2, t3, t4)
    }
}

fn test_module<F, Args, T0>(
    uut: F,
    desc: VerilogDescriptor,
    vals: impl Iterator<Item = Args>,
) -> TestModule
where
    F: Testable<Args, T0>,
    T0: Timed,
{
    let VerilogDescriptor { name, body } = desc;
    let mut num_cases = 0;
    let cases = vals
        .map(|x| {
            num_cases += 1;
            x
        })
        .map(|arg| uut.test_string(&name, arg))
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

pub struct VerilogDescriptor {
    pub name: String,
    pub body: String,
}

const VERILOG_INDENT_INCREASERS: [&str; 3] = ["begin", "function", "case"];
const VERILOG_INDENT_DECREASERS: [&str; 3] = ["end", "endfunction", "endcase"];

impl VerilogDescriptor {
    fn display(&self, mut f: impl std::io::Write, line_numbers: bool) -> std::io::Result<()> {
        // Print the verilog with line numbers
        // Indent the lines
        let mut indent = 0;
        for (i, line) in self.body.lines().enumerate() {
            if line_numbers {
                write!(f, "{:3} ", i + 1)?;
            }
            let line = line.trim();
            if line.is_empty() {
                writeln!(f)?;
                continue;
            }
            if VERILOG_INDENT_DECREASERS
                .iter()
                .any(|x| line.starts_with(x))
            {
                indent -= 1;
            }
            for _ in 0..indent {
                write!(f, "    ")?;
            }
            if VERILOG_INDENT_INCREASERS
                .iter()
                .any(|x| line.starts_with(x))
            {
                indent += 1;
            }
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
    pub fn code(&self) -> String {
        let mut s = Vec::new();
        self.display(&mut s, false).unwrap();
        String::from_utf8(s).unwrap()
    }
}

impl std::fmt::Debug for VerilogDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = Vec::new();
        self.display(&mut s, true).unwrap();
        let s = String::from_utf8(s).unwrap();
        write!(f, "{}", s)
    }
}

pub struct TestModule {
    pub testbench: String,
    pub num_cases: usize,
}

impl TestModule {
    pub fn new<F, Args, T0>(
        uut: F,
        desc: VerilogDescriptor,
        vals: impl Iterator<Item = Args>,
    ) -> TestModule
    where
        F: Testable<Args, T0>,
        T0: Timed,
    {
        test_module(uut, desc, vals)
    }
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
    let design = compile_design::<K>(crate::compiler::driver::CompilationMode::Asynchronous)?;
    let verilog = generate_verilog(&design)?;
    eprintln!("Verilog {:?}", verilog);
    let vm_inputs = vals.clone();
    let mut vm_test_count = 0;
    for input in vm_inputs {
        let args_for_vm = input.vec_tb();
        let expected = uut.apply(input).typed_bits();
        let actual = execute_function(&design, args_for_vm)?;
        if expected.bits != actual.bits {
            return Err(RHDLError::VerilogVerificationErrorTyped { expected, actual });
        }
        vm_test_count += 1;
    }
    eprintln!("VM test passed {} cases OK", vm_test_count);
    let tm = test_module(uut, verilog, vals);
    //eprintln!("{}", tm.testbench);
    tm.run_iverilog()
}

impl std::fmt::Debug for TestModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.testbench.fmt(f)
    }
}

#[cfg(feature = "iverilog")]
impl TestModule {
    pub fn run_iverilog(&self) -> Result<(), RHDLError> {
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
        for case in String::from_utf8_lossy(&output.stdout)
            .lines()
            .take(self.num_cases)
            .map(|line| line.split(' ').collect::<Vec<_>>())
        {
            let expected = case[0];
            let actual = case[1];
            if case[0] != case[1] {
                return Err(RHDLError::VerilogVerificationError {
                    expected: expected.into(),
                    got: actual.into(),
                });
            }
        }
        eprintln!("iverilog test passed {} cases OK", self.num_cases);
        Ok(())
    }
}

// This is split up so that in the future we can add additional
// test programs (verilator?) and still keep the back end in place.
#[cfg(feature = "iverilog")]
pub fn test_with_iverilog<F, Args, T0>(
    uut: F,
    desc: VerilogDescriptor,
    vals: impl Iterator<Item = Args>,
) -> Result<(), RHDLError>
where
    F: Testable<Args, T0>,
    T0: Timed,
{
    test_module(uut, desc, vals).run_iverilog()
}

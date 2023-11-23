use anyhow::Result;
use anyhow::{anyhow, bail};
use rhdl_bits::alias::{b12, b4, b8};
use rhdl_core::{Digital, Kind};
use rhdl_macro::Digital;
use std::alloc::Layout;
use std::collections::{BTreeMap, HashSet};
use std::env::args;
use std::path::Path;
use std::{collections::HashMap, fmt::Display};
use std::{default, vec};
use syn::token::In;
use utils::IndentingFormatter;
use zerocopy::AsBytes;

pub mod utils;

#[derive(Debug, Clone, PartialEq)]
struct DigitalSignature {
    arguments: Vec<Kind>,
    ret: Kind,
}

trait Describable<Args> {
    fn describe() -> DigitalSignature;
}

impl<F, T1, T2> Describable<(T1, T2)> for F
where
    F: Fn(T1) -> T2,
    T1: Digital,
    T2: Digital,
{
    fn describe() -> DigitalSignature {
        DigitalSignature {
            arguments: vec![T1::static_kind()],
            ret: T2::static_kind(),
        }
    }
}

impl<F, T1, T2, T3> Describable<(T1, T2, T3)> for F
where
    F: Fn(T1, T2) -> T3,
    T1: Digital,
    T2: Digital,
    T3: Digital,
{
    fn describe() -> DigitalSignature {
        DigitalSignature {
            arguments: vec![T1::static_kind(), T2::static_kind()],
            ret: T3::static_kind(),
        }
    }
}

fn inspect_digital<F, Args>(_f: F) -> DigitalSignature
where
    F: Describable<Args>,
{
    F::describe()
}

struct Junk<Args> {
    _args: std::marker::PhantomData<Args>,
}

impl<F, T1, T2, T3> Describable<(T1, T2, T3)> for Junk<F>
where
    F: Fn(T1, T2) -> T3,
    T1: Digital,
    T2: Digital,
    T3: Digital,
{
    fn describe() -> DigitalSignature {
        DigitalSignature {
            arguments: vec![T1::static_kind(), T2::static_kind()],
            ret: T3::static_kind(),
        }
    }
}

trait TypeName {
    fn type_name() -> String;
}

impl TypeName for usize {
    fn type_name() -> String {
        "usize".into()
    }
}

impl TypeName for String {
    fn type_name() -> String {
        "String".into()
    }
}

impl<T: TypeName> TypeName for Vec<T> {
    fn type_name() -> String {
        format!("Vec<{}>", T::type_name())
    }
}

impl TypeName for () {
    fn type_name() -> String {
        "Unit".into()
    }
}

impl<T1: TypeName, T2: TypeName> TypeName for (T1, T2) {
    fn type_name() -> String {
        format!("({}, {})", T1::type_name(), T2::type_name())
    }
}

impl TypeName for u8 {
    fn type_name() -> String {
        "u8".into()
    }
}

impl TypeName for Color {
    fn type_name() -> String {
        "Color".into()
    }
}

fn inspect_function<F, T1, T2>(_f: F) -> String
where
    F: Fn(T1) -> T2,
    T1: TypeName,
    T2: TypeName,
{
    format!("Function: {} -> {}", T1::type_name(), T2::type_name())
}

trait Hello {
    fn hello(&self) {
        println!("Hello, world!");
    }
}

#[derive(Default)]
struct Foo {
    a: i32,
    b: i32,
}

impl Hello for Foo {}

#[derive(Default, Copy, Clone, PartialEq, Digital)]
enum Color {
    #[default]
    Red,
    Green(b4),
    Yellow(b8, b12),
    Blue {
        a: b8,
        b: b12,
    },
}

trait HasAstStuff<T> {
    fn ast_stuff() -> String;
}

struct my_fancy_foo {}

/*
impl HasAstStuff for my_fancy_foo {
    fn ast_stuff() -> String {
        "my_fancy_foo".into()
    }
}
*/

struct color_green {}
struct color_yellow {}

impl<F> HasAstStuff<F> for color_green
where
    F: Fn(b4) -> Color,
{
    fn ast_stuff() -> String {
        "color_green constructor".into()
    }
}

impl<F> HasAstStuff<F> for color_yellow
where
    F: Fn(b8, b12) -> Color,
{
    fn ast_stuff() -> String {
        "color_yellow constructor".into()
    }
}

fn my_fancy_foo(len: b4) -> b8 {
    42.into()
}

trait KernelFn {
    fn kernel_fn() -> String;
}

struct shift<const N: u128> {}

impl<const N: u128> KernelFn for shift<N> {
    fn kernel_fn() -> String {
        format!("shift<{}>", N)
    }
}

fn shift<const N: u128>(x: b8) -> b8 {
    x << N
}

impl Hello for Color {}

// Logging playground.

// Need a basic macro called `vcd!` that does the following:
// 1. Takes a log level (optionally), like error, warn, info, debug, trace.
// 2. Takes a target name (optionally), like "foo", "bar", "baz".  Must be a string literal.
// 3. Takes an object to log, which must impl the Digital trait.
// Given these three, the macro should
//  1. Acquire the thread-local global logger object (which must be set up)
//  2. Call the logger's log method, passing the log level, target name, and object to log.
// The logger object must be a thread-local global, and must be set up by the user.
// The logger object must impl the Logger trait, which is defined in rhdl-core.

// For a generalized logging to work, we need both a serialization and dynamic deserialization
// mechanism.  Something like:
// 1. A Digital Object --> byte array.
// 2. Given a Kind + a byte array, query a serialized object
//

// We don't actually need the Digital Object to be a trait object.  It would be sufficient to
// convert it into a Record, and then have _that_ be the trait object.  But it really doesn't
// matter.  In the end, we want to inspect the records at run time.
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NoteLevel {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}

#[macro_export]
macro_rules! note {
    (target: $target:literal, $lvl:expr, $value:expr) => {{
        let lvl = $lvl;
        $crate::note(
            lvl,
            concat!(
                module_path!(),
                "::",
                file!(),
                "::",
                line!(),
                "::",
                column!(),
                "::",
                $target
            ),
            &$value,
        )
    }};
    // note!(level, value) --> note!(target: module_path)
    ($lvl: expr, $value: expr) => {{
        let lvl = $lvl;
        $crate::note(
            lvl,
            concat!(
                module_path!(),
                "::",
                file!(),
                "::",
                line!(),
                "::",
                column!(),
                "::",
                stringify!($value)
            ),
            &$value,
        )
    }};
}

fn note(lvl: NoteLevel, target: &'static str, value: &impl Digital) {
    eprintln!("{:?}: {} = {:?}", lvl, target, value.typed_bits());
}

fn main() {
    // Some facts about Color

    let layout = Layout::new::<Color>();
    eprintln!("Color layout: {:?}", layout);

    // Check for default via variant syntax
    let a = Color::Red; // a is Color::Red
    a.hello();
    note!(target: "b", NoteLevel::Debug, a);
    note!(NoteLevel::Info, a);
    //    note!(0, a);
    let b = Color::Green(Default::default());
    b.hello();
    let c = Color::Blue {
        a: Default::default(),
        b: Default::default(),
    };
    let d = Foo {
        ..Default::default()
    };
    c.hello();
    Hello::hello(&c);
    let sig = inspect_digital(my_fancy_foo);
    eprintln!("{:?}", sig);
    //eprintln!("{}", my_fancy_foo::ast_stuff());
    let sig = inspect_digital(Color::Green);
    eprintln!("{:?}", sig);
    let sig = inspect_digital(Color::Yellow);
    eprintln!("{:?}", sig);
    let sig = inspect_digital(shift::<3>);
    eprintln!("{:?}", sig);
    let kernel = shift::<3>::kernel_fn();
    eprintln!("{}", kernel);
}

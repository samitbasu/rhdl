use anyhow::Result;
use anyhow::{anyhow, bail};
use std::collections::{BTreeMap, HashSet};
use std::env::args;
use std::path::Path;
use std::{collections::HashMap, fmt::Display};
use std::{default, vec};
use syn::token::In;
use utils::IndentingFormatter;

pub mod utils;

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

#[derive(Default)]
enum Color {
    #[default]
    Red,
    Green(u8),
    Blue {
        a: i32,
        b: i32,
    },
}

impl Hello for Color {}

fn main() {
    // Check for default via variant syntax
    let a = Color::Red; // a is Color::Red
    a.hello();
    let b = Color::Green(Default::default());
    b.hello();
    let c = Color::Blue {
        a: Default::default(),
        b: Default::default(),
    };
    c.hello();
}

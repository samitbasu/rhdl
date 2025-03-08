use crate::constraints::Constraint;

pub mod lattice;
pub mod xilinx;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Input,
    Output,
    InOut,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub direction: Direction,
    pub width: usize,
}

pub fn port(name: &str, direction: Direction, width: usize) -> Port {
    Port {
        name: name.into(),
        direction,
        width,
    }
}

#[derive(Debug, Clone)]
pub struct Driver {
    pub ports: Vec<Port>,
    pub hdl: String,
    pub constraints: String,
}

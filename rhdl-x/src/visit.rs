use anyhow::Result;
use rhdl_core::Circuit;

pub trait Visitor {
    fn visit<C: Circuit>(&mut self, instance_name: &str, circuit: &C) -> Result<()>;
    fn push(&mut self);
    fn pop(&mut self);
}

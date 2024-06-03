use crate::{error::RHDLError, rhif::Object};

pub trait Pass {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(input: Object) -> Result<Object, RHDLError>;
}

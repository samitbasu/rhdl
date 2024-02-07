use crate::circuit::Circuit;
use anyhow::Result;

pub trait Translator {
    fn translate<C: Circuit>(&self, circuit: &C) -> Result<String>;
}

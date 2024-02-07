use crate::circuit::Circuit;
use anyhow::Result;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TranslationKind {
    Verilog,
}

pub trait Translator {
    fn kind(&self) -> TranslationKind;
    fn translate<C: Circuit>(&mut self, circuit: &C) -> Result<()>;
    fn custom_code(&mut self, code: &str) -> Result<()>;
    fn finish(self) -> Result<String>;
}

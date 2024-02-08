use crate::circuit::Circuit;
use anyhow::Result;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TranslationKind {
    Verilog,
}

pub trait Translator {
    fn kind(&self) -> TranslationKind;
    fn translate<C: Circuit>(&mut self, name: &str, circuit: &C) -> Result<()>;
    fn push(&mut self) -> Result<()>;
    fn pop(&mut self) -> Result<()>;
    fn custom_code(&mut self, name: &str, code: &str) -> Result<()>;
    fn finish(self) -> Result<String>;
}

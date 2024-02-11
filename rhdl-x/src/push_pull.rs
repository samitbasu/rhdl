use crate::{strobe::Strobe, dff::DFF};

#[derive(Clone)]
pub struct Push {
    strobe: Strobe<32>,
    stroke: DFF<bool>,
}


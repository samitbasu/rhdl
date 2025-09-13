use quote::{quote, ToTokens};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Location {
    BGABall { row: BGARow, col: u8 },
    Edge { num: usize },
    Custom { name: String },
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::BGABall { row, col } => write!(f, "{row:?}{col}"),
            Location::Edge { num } => write!(f, "{num}"),
            Location::Custom { name } => write!(f, "{name}"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BGARow {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    J,
    K,
    L,
    M,
    N,
    P,
    R,
    T,
    U,
    V,
    W,
    Y,
    AA,
    AB,
    AC,
    AD,
    AE,
    AF,
    AG,
    AH,
    AJ,
    AK,
    AL,
    AM,
    AN,
    AP,
    AR,
    AT,
    AU,
    AV,
    AW,
    AY,
    BA,
    BB,
    BC,
    BD,
    BE,
    BF,
    BG,
    BH,
    BJ,
    BK,
    BL,
    BM,
    BN,
    BP,
    BR,
    BT,
    BU,
    BV,
    BW,
    BY,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SlewType {
    Slow,
    Fast,
    Custom(String),
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum IOStandard {
    LowVoltageCMOS_1v8,
    LowVoltageCMOS_3v3,
    StubSeriesTerminatedLogic_II,
    DifferentialStubSeriesTerminatedLogic_II,
    LowVoltageDifferentialSignal_2v5,
    StubSeriesTerminatedLogic_1v5,
    LowVoltageCMOS_1v5,
    DifferentialStubSeriesTerminatedLogic_1v5,
}

impl std::fmt::Display for IOStandard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IOStandard::LowVoltageCMOS_1v5 => write!(f, "LVCMOS15"),
            IOStandard::LowVoltageCMOS_1v8 => write!(f, "LVCMOS18"),
            IOStandard::LowVoltageCMOS_3v3 => write!(f, "LVCMOS33"),
            IOStandard::StubSeriesTerminatedLogic_II => write!(f, "SSTL18_II"),
            IOStandard::DifferentialStubSeriesTerminatedLogic_II => write!(f, "DIFF_SSTL18_II"),
            IOStandard::LowVoltageDifferentialSignal_2v5 => write!(f, "LVDS_25"),
            IOStandard::StubSeriesTerminatedLogic_1v5 => write!(f, "SSTL15"),
            IOStandard::DifferentialStubSeriesTerminatedLogic_1v5 => write!(f, "DIFF_SSTL15"),
        }
    }
}

impl ToTokens for IOStandard {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = self.to_string();
        tokens.extend(quote! { #s });
    }
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Signal(IOStandard),
    Slew(SlewType),
    Location(Location),
    Custom(String),
}

#[macro_export]
macro_rules! bga_pin {
    ($row:ident, $col:expr) => {
        $crate::constraints::Location::BGABall {
            row: $crate::constraints::BGARow::$row,
            col: $col,
        }
    };
}

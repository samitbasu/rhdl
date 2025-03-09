use serde::Serialize;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Location {
    BGABall { row: BGARow, col: u8 },
    Edge { num: usize },
    Custom { name: String },
}

impl Serialize for Location {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Location::BGABall { row, col } => serializer.serialize_str(&format!("{row:?}{col}")),
            Location::Edge { num } => serializer.serialize_str(&format!("{num}")),
            Location::Custom { name } => serializer.serialize_str(name),
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
#[derive(Clone, Debug, Serialize)]
pub enum SignalType {
    #[serde(rename = "LVCMOS18")]
    LowVoltageCMOS_1v8,
    #[serde(rename = "LVCMOS33")]
    LowVoltageCMOS_3v3,
    #[serde(rename = "SSTL18_II")]
    StubSeriesTerminatedLogic_II,
    #[serde(rename = "DIFF_SSTL18_II")]
    DifferentialStubSeriesTerminatedLogic_II,
    #[serde(rename = "LVDS_25")]
    LowVoltageDifferentialSignal_2v5,
    #[serde(rename = "SSTL15")]
    StubSeriesTerminatedLogic_1v5,
    #[serde(rename = "LVCMOS15")]
    LowVoltageCMOS_1v5,
    #[serde(rename = "DIFF_SSTL15")]
    DifferentialStubSeriesTerminatedLogic_1v5,
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Signal(SignalType),
    Slew(SlewType),
    Location(Location),
    Custom(String),
}

#[derive(Clone, Debug)]
pub enum MountPoint {
    Input(std::ops::Range<usize>),
    Output(std::ops::Range<usize>),
}

impl Serialize for MountPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MountPoint::Input(range) => {
                if range.is_empty() {
                    panic!("Invalid input for Mount Point");
                }
                if range.len() == 1 {
                    serializer.serialize_str(&format!("inner_input[{}]", range.start))
                } else {
                    serializer.serialize_str(&format!(
                        "inner_input[{}:{}]",
                        range.end.saturating_sub(1),
                        range.start
                    ))
                }
            }
            MountPoint::Output(range) => {
                if range.is_empty() {
                    panic!("Invalid output for Mount Point");
                }
                if range.len() == 1 {
                    serializer.serialize_str(&format!("inner_output[{}]", range.start))
                } else {
                    serializer.serialize_str(&format!(
                        "inner_output[{}:{}]",
                        range.end.saturating_sub(1),
                        range.start
                    ))
                }
            }
        }
    }
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

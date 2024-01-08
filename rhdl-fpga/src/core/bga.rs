// Pin locations in a BGA chip
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BGAPin {
    pub row: BGARow,
    pub col: usize,
}

pub const fn bga_pin(row: BGARow, col: usize) -> BGAPin {
    BGAPin { row, col }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
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
}

impl std::fmt::Display for BGAPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.row, self.col)
    }
}

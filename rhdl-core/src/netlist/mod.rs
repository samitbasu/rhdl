// The netlist is meant to capture actual hardware components and their connections.

pub enum Signed {
    Unsigned,
    Signed,
}

pub enum CellKind {
    Add,
    Sub,
    Mul(Signed),
    Lt(Signed),
    Le(Signed),
    Eq,
    Ne,
    Ge(Signed),
    Gt(Signed),
    BitXor,
    BitAnd,
    BitOr,
    Not,
    Mux,
    Any,
    All,
    Xor,
    Zero,
    One,
    Neg,
    DynShl,
    DFF,
    ROM,
}

/*

Eq -> (a ^ b).any().not()
Lt ->

*/

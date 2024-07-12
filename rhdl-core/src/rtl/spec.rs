use crate::error::RHDLError;

#[derive(Clone, PartialEq)]
pub enum OpCode {
    // Noop
    Noop,
    // lhs <- arg1 op arg2
    Binary(Binary),
    // lhs <- op arg1
    Unary(Unary),
    // lhs <- cond ? true_value : false_value
    Select(Select),
    // lhs <- {{ r1, r2, ... }}
    Concat(Concat),
    // lhs <- arg[base_offset + arg * stride +: len]
    DynamicIndex(DynamicIndex),
    // lhs <- arg; lhs[base_offset + arg * stride +: len] <- value
    DynamicSplice(DynamicSplice),
    // lhs <- arg[bit_range]
    Index(Index),
    // lhs <- arg; lhs[bit_range] <- value
    Splice(Splice),
    // lhs <- arg
    Assign(Assign),
    // Comment
    Comment(String),
    // lhs <- table[slot]
    Case(Case),
    // lhs <- unsigned(slot)
    AsBits(Cast),
    // lhs <- signed(slot)
    AsSigned(Cast),
}

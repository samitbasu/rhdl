//! Symbol table for literals and registers.
//!
//! A symbol table consists of the following:
//!
//!  - A [SlotVec] of literals, which map [LiteralId]s to some literal value type `L` and associated metadata `M`.
//!  - A [SlotVec] of registers, which map [RegisterId]s to some register value type `R` and associated metadata `M`.
//!
//! Access to the symbol table can occur via the [Symbol] enum which can represent either a literal or a register.
//! You can also access the table directly via the [LiteralId] and [RegisterId] types, in which case, you can access
//! the literal or register values directly.
//!
//! Most of the methods of this are `pub(crate)`, but a few are `pub` so that you can
//! interrogate a symbol table for analysis purposes.
use super::slot_vec::*;
use std::hash::Hash;

/// Marker trait for indicest that will
/// This trait is used to differentiate between different kinds of symbols
/// used in RHDL.  The `NAME` associated constant is used when printing the
/// symbol out for display and debug purposes.  
pub trait SymbolKind: Copy + Ord + Hash + Default {
    /// The name of the symbol kind, e.g., if this is
    /// for operand symbols, this might be "op" or even just "o".
    /// If the index is a literal index, then it will be written out as
    /// "ol{id}", or "opl{id}", etc.
    const NAME: &'static str;
}

/// Identifier for a literal in the symbol table.
///
/// The `K` type parameter is used to differentiate between different
/// kinds of symbols, e.g., operand literals, slot literals, etc.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LiteralId<K> {
    id: u64,
    index: usize,
    marker: std::marker::PhantomData<K>,
}

impl<K: SymbolKind> SlotKey for LiteralId<K> {
    fn new(id: u64, index: usize) -> LiteralId<K> {
        LiteralId {
            id,
            index,
            marker: std::marker::PhantomData,
        }
    }
    fn id(self) -> u64 {
        self.id
    }
    fn index(self) -> usize {
        self.index
    }
}
impl<K: SymbolKind> std::fmt::Display for LiteralId<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(core::format_args!("{}{}", "l", self.index))
    }
}
impl<K: SymbolKind> std::fmt::Debug for LiteralId<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(core::format_args!("{}{}{}", K::NAME, "l", self.index))
    }
}

/// Identifier for a register in the symbol table.
/// The `K` type parameter is used to differentiate between different
/// kinds of symbols, e.g., operand registers, slot registers, etc.
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RegisterId<K>
where
    K: Hash,
    K: std::cmp::Ord,
    K: std::marker::Copy,
{
    id: u64,
    index: usize,
    marker: std::marker::PhantomData<K>,
}

impl<K: SymbolKind> SlotKey for RegisterId<K> {
    fn new(id: u64, index: usize) -> RegisterId<K> {
        RegisterId {
            id,
            index,
            marker: std::marker::PhantomData,
        }
    }
    fn id(self) -> u64 {
        self.id
    }
    fn index(self) -> usize {
        self.index
    }
}
impl<K: SymbolKind> std::fmt::Display for RegisterId<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.index)
    }
}
impl<K: SymbolKind> std::fmt::Debug for RegisterId<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}r{}", K::NAME, self.index)
    }
}

/// A symbol in the symbol table, which can be either a literal or a register.
/// The `K` type parameter is used to differentiate between different
/// kinds of symbols, e.g., operand symbols, slot symbols, etc.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Symbol<K: SymbolKind> {
    /// A literal symbol.
    Literal(LiteralId<K>),
    /// A register symbol.
    Register(RegisterId<K>),
}

impl<K: SymbolKind> std::fmt::Display for Symbol<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Literal(id) => id.fmt(f),
            Symbol::Register(id) => id.fmt(f),
        }
    }
}

impl<K: SymbolKind> std::fmt::Debug for Symbol<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(id) => id.fmt(f),
            Self::Register(id) => id.fmt(f),
        }
    }
}

impl<K: SymbolKind> From<LiteralId<K>> for Symbol<K> {
    fn from(val: LiteralId<K>) -> Symbol<K> {
        Symbol::Literal(val)
    }
}

impl<K: SymbolKind> From<RegisterId<K>> for Symbol<K> {
    fn from(val: RegisterId<K>) -> Symbol<K> {
        Symbol::Register(val)
    }
}

impl<K: SymbolKind> From<&LiteralId<K>> for Symbol<K> {
    fn from(val: &LiteralId<K>) -> Symbol<K> {
        Symbol::Literal(*val)
    }
}

impl<K: SymbolKind> From<&RegisterId<K>> for Symbol<K> {
    fn from(val: &RegisterId<K>) -> Symbol<K> {
        Symbol::Register(*val)
    }
}

impl<K: SymbolKind> Symbol<K> {
    /// Returns the literal ID if the symbol is a literal.
    pub fn lit(self) -> Option<LiteralId<K>> {
        match self {
            Symbol::Literal(lit) => Some(lit),
            _ => None,
        }
    }
    /// Returns the register ID if the symbol is a register.
    pub fn reg(self) -> Option<RegisterId<K>> {
        match self {
            Symbol::Register(reg) => Some(reg),
            _ => None,
        }
    }
    /// Returns true if the symbol is a literal.
    #[must_use]
    pub fn is_lit(&self) -> bool {
        matches!(self, Symbol::Literal(_))
    }
    /// Returns true if the symbol is a register.
    #[must_use]
    pub fn is_reg(&self) -> bool {
        matches!(self, Symbol::Register(_))
    }
}

/// A symbol table mapping literals and registers to their values and metadata.
///
/// The `L` type parameter is the type of literal values.
/// The `R` type parameter is the type of register values.
/// The `M` type parameter is the type of metadata associated with each symbol.
/// The `K` type parameter is used to differentiate between different kinds of symbols.
///
/// The `K` type is a marker used so that literal and register IDs from different
/// symbol tables are not confused with each other.
///
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SymbolTable<L, R, M, K: SymbolKind> {
    lit: SlotVec<(L, M), LiteralId<K>>,
    reg: SlotVec<(R, M), RegisterId<K>>,
}

type Parts<L, R, M, K> = (
    SlotVec<(L, M), LiteralId<K>>,
    SlotVec<(R, M), RegisterId<K>>,
);

impl<L, R, M, K: SymbolKind> Default for SymbolTable<L, R, M, K> {
    fn default() -> Self {
        Self {
            lit: SlotVec::default(),
            reg: SlotVec::default(),
        }
    }
}

impl<L, R, M, K: SymbolKind> SymbolTable<L, R, M, K> {
    /// Returns a reference to the vector of literal values and their metadata.
    #[must_use]
    pub fn lit_vec(&self) -> &[(L, M)] {
        self.lit.inner()
    }
    /// Returns a reference to the vector of register values and their metadata.
    #[must_use]
    pub fn reg_vec(&self) -> &[(R, M)] {
        self.reg.inner()
    }
    pub(crate) fn lit(&mut self, value: L, meta: M) -> Symbol<K> {
        let lid = self.lit.push((value, meta));
        Symbol::Literal(lid)
    }
    pub(crate) fn reg(&mut self, value: R, meta: M) -> Symbol<K> {
        let rid = self.reg.push((value, meta));
        Symbol::Register(rid)
    }
    /// Iterate over all literals in the symbol table.
    pub fn iter_lit(&self) -> impl Iterator<Item = (LiteralId<K>, &(L, M))> + '_ {
        self.lit.iter()
    }
    /// Iterate over all registers in the symbol table.
    pub fn iter_reg(&self) -> impl Iterator<Item = (RegisterId<K>, &(R, M))> + '_ {
        self.reg.iter()
    }
    /// Iterate over all symbols in the symbol table.
    pub fn iter_sym(&self) -> impl Iterator<Item = (Symbol<K>, &M)> + '_ {
        self.iter_lit()
            .map(|(lid, (_, meta))| (Symbol::Literal(lid), meta))
            .chain(
                self.iter_reg()
                    .map(|(rid, (_, meta))| (Symbol::Register(rid), meta)),
            )
    }
    pub(crate) fn iter_lit_mut(
        &mut self,
    ) -> impl Iterator<Item = (LiteralId<K>, &mut (L, M))> + '_ {
        self.lit.iter_mut()
    }
    pub(crate) fn merge(
        &mut self,
        other: Self,
    ) -> impl Fn(Symbol<K>) -> Symbol<K> + use<L, R, M, K> {
        let Self { lit, reg } = other;
        let lit_merge = self.lit.merge(lit);
        let reg_merge = self.reg.merge(reg);
        move |symbol| match symbol {
            Symbol::Literal(id) => Symbol::Literal(lit_merge(id)),
            Symbol::Register(id) => Symbol::Register(reg_merge(id)),
        }
    }
    pub(crate) fn retain<F: Fn(Symbol<K>, &M) -> bool + Clone>(
        &mut self,
        f: F,
    ) -> impl Fn(Symbol<K>) -> Option<Symbol<K>> + use<F, L, R, M, K> {
        let f_lit = f.clone();
        let f_reg = f;
        let retain_lit = self
            .lit
            .retain(move |id, x| f_lit(Symbol::Literal(id), &x.1));
        let retain_reg = self
            .reg
            .retain(move |id, x| f_reg(Symbol::Register(id), &x.1));
        move |symbol| match symbol {
            Symbol::Literal(lid) => retain_lit(lid).map(Symbol::Literal),
            Symbol::Register(rid) => retain_reg(rid).map(Symbol::Register),
        }
    }
    pub(crate) fn transmute<T, F>(self, f: F) -> SymbolTable<L, R, T, K>
    where
        F: Fn(Symbol<K>, M) -> T,
    {
        SymbolTable {
            lit: self
                .lit
                .transmute(|indx, val| (val.0, f(Symbol::Literal(indx), val.1))),
            reg: self
                .reg
                .transmute(|indx, val| (val.0, f(Symbol::Register(indx), val.1))),
        }
    }
    pub(crate) fn into_parts(self) -> Parts<L, R, M, K> {
        (self.lit, self.reg)
    }
    pub(crate) fn from_parts(
        lit: SlotVec<(L, M), LiteralId<K>>,
        reg: SlotVec<(R, M), RegisterId<K>>,
    ) -> Self {
        Self { lit, reg }
    }
    /// Check if a symbol key is valid in the symbol table.
    #[must_use]
    pub fn is_key_valid(&self, key: Symbol<K>) -> bool {
        match key {
            Symbol::Literal(lid) => self.lit.is_key_valid(lid),
            Symbol::Register(rid) => self.reg.is_key_valid(rid),
        }
    }
}

impl<L, R, M, K: SymbolKind> std::ops::Index<&Symbol<K>> for SymbolTable<L, R, M, K> {
    type Output = M;
    fn index(&self, index: &Symbol<K>) -> &Self::Output {
        self.index(*index)
    }
}

impl<L, R, M, K: SymbolKind> std::ops::IndexMut<&Symbol<K>> for SymbolTable<L, R, M, K> {
    fn index_mut(&mut self, index: &Symbol<K>) -> &mut Self::Output {
        self.index_mut(*index)
    }
}

impl<L, R, M, K: SymbolKind> std::ops::Index<Symbol<K>> for SymbolTable<L, R, M, K> {
    type Output = M;

    fn index(&self, index: Symbol<K>) -> &Self::Output {
        match index {
            Symbol::Literal(id) => &self.lit[id].1,
            Symbol::Register(id) => &self.reg[id].1,
        }
    }
}

impl<L, R, M, K: SymbolKind> std::ops::IndexMut<Symbol<K>> for SymbolTable<L, R, M, K> {
    fn index_mut(&mut self, index: Symbol<K>) -> &mut Self::Output {
        match index {
            Symbol::Literal(id) => &mut self.lit[id].1,
            Symbol::Register(id) => &mut self.reg[id].1,
        }
    }
}

impl<L, R, M, K: SymbolKind> std::ops::Index<LiteralId<K>> for SymbolTable<L, R, M, K> {
    type Output = L;

    fn index(&self, index: LiteralId<K>) -> &Self::Output {
        &self.lit[index].0
    }
}

impl<L, R, M, K: SymbolKind> std::ops::IndexMut<LiteralId<K>> for SymbolTable<L, R, M, K> {
    fn index_mut(&mut self, index: LiteralId<K>) -> &mut Self::Output {
        &mut self.lit[index].0
    }
}

impl<L, R, M, K: SymbolKind> std::ops::Index<&LiteralId<K>> for SymbolTable<L, R, M, K> {
    type Output = L;

    fn index(&self, index: &LiteralId<K>) -> &Self::Output {
        &self.lit[index].0
    }
}

impl<L, R, M, K: SymbolKind> std::ops::IndexMut<&LiteralId<K>> for SymbolTable<L, R, M, K> {
    fn index_mut(&mut self, index: &LiteralId<K>) -> &mut Self::Output {
        &mut self.lit[index].0
    }
}

impl<L, R, M, K: SymbolKind> std::ops::Index<RegisterId<K>> for SymbolTable<L, R, M, K> {
    type Output = R;

    fn index(&self, index: RegisterId<K>) -> &Self::Output {
        &self.reg[index].0
    }
}

impl<L, R, M, K: SymbolKind> std::ops::IndexMut<RegisterId<K>> for SymbolTable<L, R, M, K> {
    fn index_mut(&mut self, index: RegisterId<K>) -> &mut Self::Output {
        &mut self.reg[index].0
    }
}

impl<L, R, M, K: SymbolKind> std::ops::Index<&RegisterId<K>> for SymbolTable<L, R, M, K> {
    type Output = R;

    fn index(&self, index: &RegisterId<K>) -> &Self::Output {
        &self.reg[index].0
    }
}

impl<L, R, M, K: SymbolKind> std::ops::IndexMut<&RegisterId<K>> for SymbolTable<L, R, M, K> {
    fn index_mut(&mut self, index: &RegisterId<K>) -> &mut Self::Output {
        &mut self.reg[index].0
    }
}

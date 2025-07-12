use super::slot_vec::*;
use std::hash::Hash;

pub trait SymbolKind: Copy + Ord + Hash + Default {
    const NAME: &'static str;
}

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
        write!(f, "{}{}", "r", self.index)
    }
}
impl<K: SymbolKind> std::fmt::Debug for RegisterId<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", K::NAME, "r", self.index)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Symbol<K: SymbolKind> {
    Literal(LiteralId<K>),
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

impl<K: SymbolKind> Symbol<K> {
    pub fn lit(self) -> Option<LiteralId<K>> {
        match self {
            Symbol::Literal(lit) => Some(lit),
            _ => None,
        }
    }
    pub fn reg(self) -> Option<RegisterId<K>> {
        match self {
            Symbol::Register(reg) => Some(reg),
            _ => None,
        }
    }
    pub fn is_lit(&self) -> bool {
        matches!(self, Symbol::Literal(_))
    }
    pub fn is_reg(&self) -> bool {
        matches!(self, Symbol::Register(_))
    }
}

#[derive(Clone, Hash)]
pub struct SymbolTable<L, R, M, K: SymbolKind> {
    lit: SlotVec<(L, M), LiteralId<K>>,
    reg: SlotVec<(R, M), RegisterId<K>>,
}

impl<L, R, M, K: SymbolKind> Default for SymbolTable<L, R, M, K> {
    fn default() -> Self {
        Self {
            lit: SlotVec::default(),
            reg: SlotVec::default(),
        }
    }
}

impl<L, R, M, K: SymbolKind> SymbolTable<L, R, M, K> {
    pub fn lit_vec(&self) -> &[(L, M)] {
        self.lit.inner()
    }
    pub fn reg_vec(&self) -> &[(R, M)] {
        self.reg.inner()
    }
    pub fn lit(&mut self, value: L, meta: M) -> Symbol<K> {
        let lid = self.lit.push((value, meta));
        Symbol::Literal(lid)
    }
    pub fn reg(&mut self, value: R, meta: M) -> Symbol<K> {
        let rid = self.reg.push((value, meta));
        Symbol::Register(rid)
    }
    pub fn iter_lit(&self) -> impl Iterator<Item = (LiteralId<K>, &(L, M))> + '_ {
        self.lit.iter()
    }
    pub fn iter_reg(&self) -> impl Iterator<Item = (RegisterId<K>, &(R, M))> + '_ {
        self.reg.iter()
    }
    pub fn iter_sym(&self) -> impl Iterator<Item = (Symbol<K>, &M)> + '_ {
        self.iter_lit()
            .map(|(lid, (_, meta))| (Symbol::Literal(lid), meta))
            .chain(
                self.iter_reg()
                    .map(|(rid, (_, meta))| (Symbol::Register(rid), meta)),
            )
    }
    pub fn iter_lit_mut(&mut self) -> impl Iterator<Item = (LiteralId<K>, &mut (L, M))> + '_ {
        self.lit.iter_mut()
    }
    pub fn iter_reg_mut(&mut self) -> impl Iterator<Item = (RegisterId<K>, &mut (R, M))> + '_ {
        self.reg.iter_mut()
    }
    pub fn merge(&mut self, other: Self) -> impl Fn(Symbol<K>) -> Symbol<K> + use<L, R, M, K> {
        let Self { lit, reg } = other;
        let lit_merge = self.lit.merge(lit);
        let reg_merge = self.reg.merge(reg);
        move |symbol| match symbol {
            Symbol::Literal(id) => Symbol::Literal(lit_merge(id)),
            Symbol::Register(id) => Symbol::Register(reg_merge(id)),
        }
    }
    pub fn retain_literals<F: Fn(LiteralId<K>, &(L, M)) -> bool>(
        &mut self,
        f: F,
    ) -> impl Fn(LiteralId<K>) -> Option<LiteralId<K>> + use<F, L, R, M, K> {
        self.lit.retain(f)
    }
    pub fn retain_registers<F: Fn(RegisterId<K>, &(R, M)) -> bool>(
        &mut self,
        f: F,
    ) -> impl Fn(RegisterId<K>) -> Option<RegisterId<K>> + use<F, L, R, M, K> {
        self.reg.retain(f)
    }
    pub fn retain<F: Fn(Symbol<K>, &M) -> bool + Clone>(
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
    pub fn transmute<T, F>(self, f: F) -> SymbolTable<L, R, T, K>
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
    pub fn into_parts(
        self,
    ) -> (
        SlotVec<(L, M), LiteralId<K>>,
        SlotVec<(R, M), RegisterId<K>>,
    ) {
        (self.lit, self.reg)
    }
    pub fn from_parts(
        lit: SlotVec<(L, M), LiteralId<K>>,
        reg: SlotVec<(R, M), RegisterId<K>>,
    ) -> Self {
        Self { lit, reg }
    }
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

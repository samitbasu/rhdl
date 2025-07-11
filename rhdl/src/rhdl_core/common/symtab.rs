use super::slot_vec::*;

new_key_type!(LiteralId, "l");

new_key_type!(RegisterId, "r");

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Symbol {
    Literal(LiteralId),
    Register(RegisterId),
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Literal(id) => id.fmt(f),
            Symbol::Register(id) => id.fmt(f),
        }
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(id) => id.fmt(f),
            Self::Register(id) => id.fmt(f),
        }
    }
}

impl From<LiteralId> for Symbol {
    fn from(val: LiteralId) -> Symbol {
        Symbol::Literal(val)
    }
}

impl From<RegisterId> for Symbol {
    fn from(val: RegisterId) -> Symbol {
        Symbol::Register(val)
    }
}

impl Symbol {
    pub fn lit(self) -> Option<LiteralId> {
        match self {
            Symbol::Literal(lit) => Some(lit),
            _ => None,
        }
    }
    pub fn reg(self) -> Option<RegisterId> {
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
pub struct SymbolTable<L, R, M> {
    lit: SlotVec<(L, M), LiteralId>,
    reg: SlotVec<(R, M), RegisterId>,
}

impl<L, R, M> Default for SymbolTable<L, R, M> {
    fn default() -> Self {
        Self {
            lit: SlotVec::default(),
            reg: SlotVec::default(),
        }
    }
}

impl<L, R, M> SymbolTable<L, R, M> {
    pub fn lit_vec(&self) -> &[(L, M)] {
        self.lit.inner()
    }
    pub fn reg_vec(&self) -> &[(R, M)] {
        self.reg.inner()
    }
    pub fn lit(&mut self, value: L, meta: M) -> Symbol {
        let lid = self.lit.push((value, meta));
        Symbol::Literal(lid)
    }
    pub fn reg(&mut self, value: R, meta: M) -> Symbol {
        let rid = self.reg.push((value, meta));
        Symbol::Register(rid)
    }
    pub fn iter_lit(&self) -> impl Iterator<Item = (LiteralId, &(L, M))> + '_ {
        self.lit.iter()
    }
    pub fn iter_reg(&self) -> impl Iterator<Item = (RegisterId, &(R, M))> + '_ {
        self.reg.iter()
    }
    pub fn iter_sym(&self) -> impl Iterator<Item = (Symbol, &M)> + '_ {
        self.iter_lit()
            .map(|(lid, (_, meta))| (Symbol::Literal(lid), meta))
            .chain(
                self.iter_reg()
                    .map(|(rid, (_, meta))| (Symbol::Register(rid), meta)),
            )
    }
    pub fn iter_lit_mut(&mut self) -> impl Iterator<Item = (LiteralId, &mut (L, M))> + '_ {
        self.lit.iter_mut()
    }
    pub fn iter_reg_mut(&mut self) -> impl Iterator<Item = (RegisterId, &mut (R, M))> + '_ {
        self.reg.iter_mut()
    }
    pub fn merge(&mut self, other: Self) -> impl Fn(Symbol) -> Symbol + use<L, R, M> {
        let Self { lit, reg } = other;
        let lit_merge = self.lit.merge(lit);
        let reg_merge = self.reg.merge(reg);
        move |symbol| match symbol {
            Symbol::Literal(id) => Symbol::Literal(lit_merge(id)),
            Symbol::Register(id) => Symbol::Register(reg_merge(id)),
        }
    }
    pub fn retain_literals<F: Fn(LiteralId, &(L, M)) -> bool>(
        &mut self,
        f: F,
    ) -> impl Fn(LiteralId) -> Option<LiteralId> + use<F, L, R, M> {
        self.lit.retain(f)
    }
    pub fn retain_registers<F: Fn(RegisterId, &(R, M)) -> bool>(
        &mut self,
        f: F,
    ) -> impl Fn(RegisterId) -> Option<RegisterId> + use<F, L, R, M> {
        self.reg.retain(f)
    }
    pub fn retain<F: Fn(Symbol, &M) -> bool + Clone>(
        &mut self,
        f: F,
    ) -> impl Fn(Symbol) -> Option<Symbol> + use<F, L, R, M> {
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
    pub fn transmute<T, F>(self, f: F) -> SymbolTable<L, R, T>
    where
        F: Fn(Symbol, M) -> T,
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
    pub fn into_parts(self) -> (SlotVec<(L, M), LiteralId>, SlotVec<(R, M), RegisterId>) {
        (self.lit, self.reg)
    }
    pub fn from_parts(lit: SlotVec<(L, M), LiteralId>, reg: SlotVec<(R, M), RegisterId>) -> Self {
        Self { lit, reg }
    }
    pub fn is_key_valid(&self, key: Symbol) -> bool {
        match key {
            Symbol::Literal(lid) => self.lit.is_key_valid(lid),
            Symbol::Register(rid) => self.reg.is_key_valid(rid),
        }
    }
}

impl<L, R, M> std::ops::Index<&Symbol> for SymbolTable<L, R, M> {
    type Output = M;
    fn index(&self, index: &Symbol) -> &Self::Output {
        self.index(*index)
    }
}

impl<L, R, M> std::ops::IndexMut<&Symbol> for SymbolTable<L, R, M> {
    fn index_mut(&mut self, index: &Symbol) -> &mut Self::Output {
        self.index_mut(*index)
    }
}

impl<L, R, M> std::ops::Index<Symbol> for SymbolTable<L, R, M> {
    type Output = M;

    fn index(&self, index: Symbol) -> &Self::Output {
        match index {
            Symbol::Literal(id) => &self.lit[id].1,
            Symbol::Register(id) => &self.reg[id].1,
        }
    }
}

impl<L, R, M> std::ops::IndexMut<Symbol> for SymbolTable<L, R, M> {
    fn index_mut(&mut self, index: Symbol) -> &mut Self::Output {
        match index {
            Symbol::Literal(id) => &mut self.lit[id].1,
            Symbol::Register(id) => &mut self.reg[id].1,
        }
    }
}

impl<L, R, M> std::ops::Index<LiteralId> for SymbolTable<L, R, M> {
    type Output = L;

    fn index(&self, index: LiteralId) -> &Self::Output {
        &self.lit[index].0
    }
}

impl<L, R, M> std::ops::IndexMut<LiteralId> for SymbolTable<L, R, M> {
    fn index_mut(&mut self, index: LiteralId) -> &mut Self::Output {
        &mut self.lit[index].0
    }
}

impl<L, R, M> std::ops::Index<&LiteralId> for SymbolTable<L, R, M> {
    type Output = L;

    fn index(&self, index: &LiteralId) -> &Self::Output {
        &self.lit[index].0
    }
}

impl<L, R, M> std::ops::IndexMut<&LiteralId> for SymbolTable<L, R, M> {
    fn index_mut(&mut self, index: &LiteralId) -> &mut Self::Output {
        &mut self.lit[index].0
    }
}

impl<L, R, M> std::ops::Index<RegisterId> for SymbolTable<L, R, M> {
    type Output = R;

    fn index(&self, index: RegisterId) -> &Self::Output {
        &self.reg[index].0
    }
}

impl<L, R, M> std::ops::IndexMut<RegisterId> for SymbolTable<L, R, M> {
    fn index_mut(&mut self, index: RegisterId) -> &mut Self::Output {
        &mut self.reg[index].0
    }
}

impl<L, R, M> std::ops::Index<&RegisterId> for SymbolTable<L, R, M> {
    type Output = R;

    fn index(&self, index: &RegisterId) -> &Self::Output {
        &self.reg[index].0
    }
}

impl<L, R, M> std::ops::IndexMut<&RegisterId> for SymbolTable<L, R, M> {
    fn index_mut(&mut self, index: &RegisterId) -> &mut Self::Output {
        &mut self.reg[index].0
    }
}

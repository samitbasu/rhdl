use std::hash::Hash;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct LiteralId(usize);

impl std::fmt::Display for LiteralId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "l{}", self.0)
    }
}

impl std::fmt::Debug for LiteralId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct RegisterId(usize);

impl std::fmt::Display for RegisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

impl std::fmt::Debug for RegisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Symbol {
    Literal(LiteralId),
    Register(RegisterId),
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Literal(id) => write!(f, "{id}"),
            Symbol::Register(id) => write!(f, "{id}"),
        }
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
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
    pub fn literal(self) -> Option<LiteralId> {
        match self {
            Symbol::Literal(lit) => Some(lit),
            _ => None,
        }
    }
    pub fn register(self) -> Option<RegisterId> {
        match self {
            Symbol::Register(reg) => Some(reg),
            _ => None,
        }
    }
    pub fn is_literal(&self) -> bool {
        matches!(self, Symbol::Literal(_))
    }
    pub fn is_register(&self) -> bool {
        matches!(self, Symbol::Register(_))
    }
}

#[derive(Clone, Debug)]
pub struct SymbolTable<L, R, M> {
    literals: Vec<(L, M)>,
    registers: Vec<(R, M)>,
}

impl<L, R, M> Default for SymbolTable<L, R, M> {
    fn default() -> Self {
        Self {
            literals: Vec::default(),
            registers: Vec::default(),
        }
    }
}

impl<L, R, M> SymbolTable<L, R, M> {
    pub fn allocate_literal(&mut self, value: L, meta: M) -> Symbol {
        self.literals.push((value, meta));
        Symbol::Literal(LiteralId(self.literals.len() - 1))
    }
    pub fn allocate_register(&mut self, value: R, meta: M) -> Symbol {
        self.registers.push((value, meta));
        Symbol::Register(RegisterId(self.registers.len() - 1))
    }
    pub fn get_meta_data(&self, symbol: Symbol) -> &M {
        match symbol {
            Symbol::Literal(id) => &self.literals[id.0].1,
            Symbol::Register(id) => &self.registers[id.0].1,
        }
    }
    pub fn set_meta_data(&mut self, symbol: Symbol, meta: M) {
        match symbol {
            Symbol::Literal(id) => {
                self.literals[id.0].1 = meta;
            }
            Symbol::Register(id) => {
                self.registers[id.0].1 = meta;
            }
        }
    }
    pub fn get_literal(&self, id: LiteralId) -> &L {
        &self.literals[id.0].0
    }
    pub fn set_literal(&mut self, id: LiteralId, value: L) {
        self.literals[id.0].0 = value;
    }
    pub fn get_register(&self, id: RegisterId) -> &R {
        &self.registers[id.0].0
    }
    pub fn set_register(&mut self, id: RegisterId, value: R) {
        self.registers[id.0].0 = value;
    }
    pub fn merge(&mut self, mut other: Self) -> impl Fn(Symbol) -> Symbol {
        let literal_offset = self.literals.len();
        let register_offset = self.registers.len();
        self.literals.append(&mut other.literals);
        self.registers.append(&mut other.registers);
        move |symbol| match symbol {
            Symbol::Literal(id) => Symbol::Literal(LiteralId(id.0 + literal_offset)),
            Symbol::Register(id) => Symbol::Register(RegisterId(id.0 + register_offset)),
        }
    }
    pub fn literals(&self) -> impl Iterator<Item = (LiteralId, &L, &M)> + '_ {
        self.literals
            .iter()
            .enumerate()
            .map(|(id, val)| (LiteralId(id), &val.0, &val.1))
    }
    pub fn registers(&self) -> impl Iterator<Item = (RegisterId, &R, &M)> + '_ {
        self.registers
            .iter()
            .enumerate()
            .map(|(id, val)| (RegisterId(id), &val.0, &val.1))
    }
}

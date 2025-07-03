//! An allocator is used at various stages within RHDL.  The basic functions are:
//! The allocator keeps track of two sets of IDs, one for literals, and one for registers.
//! The allocator can allocate new IDs for each, given some data about the value, including
//! the kind/size/source-location, etc.
//! The allocator can produce a symbol table with the meta data for the values.
//! The allocator can wrap an existing symbol table to extend it.
//! The allocator can reuse literal values and provides lookup services.
//!
//! For RHIF, an allocator needs to map
//! LiteralId -> ExprLit,
//!
//! For RTL, an allocator needs to map
//! LiteralId -> BitString,
//!
//! For NTL, an allocator needs to map
//! LiteralId -> BitX
//!
//! Thus, the type of data stored for the LiteralId is generic.  For the registers, we do not
//! store any data for the register (value).  So we keep only meta data for registers.  
//!
//! RegisterId -> ()
//!
//! For each Id (register or literal), we need to know:
//!    - SourceLocation,
//!    - Name (optional)
//!    - Kind
//!    - Bit Index (only in the case of NTL)
//!
//! This means the type information (either Kind or KindAndBit) must also generic.  However,
//! we can reuse a single allocator table for literals and registers if we make the
//! index type int-like, and use 2 allocators, one for literals, and one for registers.

use internment::ArcIntern;
use std::collections::BTreeMap;
use std::hash::Hash;

use crate::rhdl_core::ast::source::source_location::SourceLocation;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Metadata<V, K> {
    value: V,
    kind: K,
    name: Option<String>,
    location: Option<SourceLocation>,
}

#[derive(Default, Clone, Debug)]
pub struct Allocator<I, V, K>
where
    V: Eq + Hash + Send + Sync + 'static,
    K: Eq + Hash + Send + Sync + 'static,
{
    values: BTreeMap<I, ArcIntern<Metadata<V, K>>>,
}

pub trait AllocatorKey: std::cmp::Ord + Copy {
    fn next(self) -> Self;
    fn first() -> Self;
}

impl<I, V, K> Allocator<I, V, K>
where
    I: AllocatorKey,
    V: Eq + Hash + Send + Sync + 'static,
    K: Eq + Hash + Send + Sync + 'static,
{
    pub fn new(values: BTreeMap<I, ArcIntern<Metadata<V, K>>>) -> Self {
        Self { values }
    }
    fn alloc_inner(&mut self, metadata: ArcIntern<Metadata<V, K>>) -> I {
        let index = if let Some((i, _a)) = self.values.last_key_value() {
            i.next()
        } else {
            I::first()
        };
        self.values.insert(index, metadata);
        index
    }
    pub fn allocate(
        &mut self,
        value: V,
        kind: K,
        name: Option<String>,
        location: Option<SourceLocation>,
    ) -> I {
        self.alloc_inner(ArcIntern::new(Metadata {
            value,
            kind,
            name,
            location,
        }))
    }
    pub fn allocate_anonymous(&mut self, value: V, kind: K) -> I {
        self.allocate(value, kind, None, None)
    }
    pub fn reuse(
        &mut self,
        value: V,
        kind: K,
        name: Option<String>,
        location: Option<SourceLocation>,
    ) -> I {
        let allocation = ArcIntern::new(Metadata {
            value,
            kind,
            name,
            location,
        });
        if let Some((i, _)) = self.values.iter().find(|(_, k)| **k == allocation) {
            *i
        } else {
            self.alloc_inner(allocation)
        }
    }
    pub fn take(self) -> BTreeMap<I, ArcIntern<Metadata<V, K>>> {
        self.values
    }
}

impl AllocatorKey for u32 {
    fn next(self) -> Self {
        self + 1
    }

    fn first() -> Self {
        0
    }
}

impl AllocatorKey for usize {
    fn next(self) -> Self {
        self + 1
    }

    fn first() -> Self {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_works() {
        let mut alloc = Allocator::<u32, (), ()>::default();
        let r0 = alloc.allocate((), (), None, None);
        assert_eq!(r0, 0);
        let r1 = alloc.allocate((), (), None, None);
        assert_eq!(r1, 1);
        let r2 = alloc.reuse((), (), None, None);
        assert_eq!(r2, 0);
    }
}

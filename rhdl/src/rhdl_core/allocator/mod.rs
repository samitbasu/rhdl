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

use std::collections::BTreeMap;

use crate::rhdl_core::ast::source::source_location::SourceLocation;

pub struct Allocation<V, K> {
    value: V,
    kind: K,
    name: Option<String>,
    location: SourceLocation,
}
pub struct Allocator<I, V, K> {
    values: BTreeMap<I, Allocation<V, K>>,
}

pub trait AllocatorKey: std::cmp::Ord + Copy {
    fn next(self) -> Self;
    fn first() -> Self;
}

impl<I, V, K> Allocator<I, V, K>
where
    I: AllocatorKey,
{
    pub fn allocate(
        &mut self,
        value: V,
        kind: K,
        name: Option<String>,
        location: SourceLocation,
    ) -> I {
        let index = if let Some((i, _a)) = self.values.last_key_value() {
            i.next()
        } else {
            I::first()
        };
        self.values.insert(
            index,
            Allocation {
                value,
                kind,
                name,
                location,
            },
        );
        index
    }
    pub fn reuse(
        &mut self,
        value: V,
        kind: K,
        name: Option<String>,
        location: SourceLocation,
    ) -> I {
        let allocation = Allocation {value, kind, name, location};
        if let Some(index) = self.values.iter().find_map(|(key, alloc)
    ) {
        
        }
    }
}

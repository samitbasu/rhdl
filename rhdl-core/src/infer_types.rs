use std::collections::HashMap;

use crate::{
    ast,
    ty::{ty_bits, Bits, TypeId},
    visit::{walk_block, Visitor},
};
use anyhow::bail;
use anyhow::Result;
use rhdl_bits::bits;
type Term = crate::ty::Ty;
type TermMap = crate::ty::TyMap;
use crate::ty::ty_as_ref as as_ref;
use crate::ty::ty_enum;
use crate::ty::ty_struct;
use crate::ty::ty_tuple as tuple;
use crate::ty::ty_var as var;

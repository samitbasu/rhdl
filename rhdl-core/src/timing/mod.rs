// We use recursion within a single object, but not between objects.
use std::ops::Range;

use crate::{ast::ast_impl::FunctionId, rtl::object::Object};

struct TimingWork<'a> {
    fn_id: FunctionId,
    module: &'a Object,
    bit_range: Range<usize>,
}

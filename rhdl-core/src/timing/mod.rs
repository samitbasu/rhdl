// We use recursion within a single object, but not between objects.
use std::ops::Range;

use crate::{ast::ast_impl::FunctionId, rtl::module::Module};

struct TimingWork<'a> {
    fn_id: FunctionId,
    module: &'a Module,
    bit_range: Range<usize>,
}

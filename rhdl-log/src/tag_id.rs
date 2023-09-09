use std::marker::PhantomData;

use crate::loggable::Loggable;

/// The [TagID] is a unique opaque identifier for a [Loggable] type.
/// For performance reasons, you will need to include a [TagID] in your
/// design for each value you want to record to the log (i.e., there is
/// no automatic logging of values).  In `rust-hdl`, you did not need
/// to think about this, because the runtime automatically logged
/// every signal in the design to the log.  However, in practice, there
/// are many signals that are not interesting to log, and the runtime
/// logging was a significant performance bottleneck.  By requiring
/// the user to explicitly specify which values to log, we can avoid
/// logging uninteresting values, and we can also avoid the overhead
/// of logging values that are not needed for debugging.  
///
/// This makes the design slightly more verbose, but also more transparent,
/// and significantly more performant to simulate.  The [TagID] is typed,
/// to help avoid misuse (although it is not foolproof).  Each [TagID] will
/// map to a signal in the resulting log, and the type of the [TagID] will
/// determine the type of the signal in the log.  For example, a [TagID] of
/// type [TagID]<Bits<8>> will map to a signal of type `Bits<8>` in the log.
///
/// Structured types (such as structs, tuples, enums, etc) are supported
/// as well.
#[derive(Debug, Clone, Copy)]
pub struct TagID<T: Loggable> {
    pub context: usize,
    pub id: usize,
    pub _marker: PhantomData<*const T>,
}

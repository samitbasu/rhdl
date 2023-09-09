use rhdl_bits::{Bits, SignedBits};

use crate::{log_builder::LogBuilder, logger::LoggerImpl, tag_id::TagID};

/// The `Loggable` trait is implemented by all types that can be logged.
/// For a type to be logged, it must have a fairly straightforward representation
/// so that it can be recorded in a file that can then be handled by other
/// tools (like <https://gtkwave.sourceforge.net/>).  For performance reasons,
/// the process of logging values is done in two stages.  First a `Loggable`
/// allocates space for itself in the log.  Then, it can record itself
/// into the log.  In almost all cases, the implementation of this trait
/// is handled by a derive macro, and you will not need to worry about it.
pub trait Loggable: Sized + Copy + Clone + Default + PartialEq {
    /// Allocate space for this `Loggable` in the log.
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder);
    /// Record this `Loggable` in the log.
    fn record<L: Loggable>(&self, tag: TagID<L>, logger: impl LoggerImpl);
}

impl Loggable for bool {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, 1);
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
        logger.write_bool(tag, *self);
    }
}

impl<const N: usize> Loggable for Bits<N> {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, N);
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
        logger.write_bits(tag, self.raw());
    }
}

impl<const N: usize> Loggable for SignedBits<N> {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, N);
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
        logger.write_signed(tag, self.raw());
    }
}

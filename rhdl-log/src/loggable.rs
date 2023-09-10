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
pub trait Loggable: Sized + Copy + Clone + PartialEq {
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
        logger.write_bits(tag, self.as_unsigned().raw());
    }
}

impl<S: Loggable, T: Loggable> Loggable for (S, T) {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        S::allocate(tag, builder.namespace("0"));
        T::allocate(tag, builder.namespace("1"));
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
    }
}

impl<S: Loggable, T: Loggable, U: Loggable> Loggable for (S, T, U) {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        S::allocate(tag, builder.namespace("0"));
        T::allocate(tag, builder.namespace("1"));
        U::allocate(tag, builder.namespace("2"));
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
        self.2.record(tag, &mut logger);
    }
}

impl<S: Loggable, T: Loggable, U: Loggable, V: Loggable> Loggable for (S, T, U, V) {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        S::allocate(tag, builder.namespace("0"));
        T::allocate(tag, builder.namespace("1"));
        U::allocate(tag, builder.namespace("2"));
        V::allocate(tag, builder.namespace("3"));
    }

    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl LoggerImpl) {
        self.0.record(tag, &mut logger);
        self.1.record(tag, &mut logger);
        self.2.record(tag, &mut logger);
        self.3.record(tag, &mut logger);
    }
}

impl<L: Loggable, const N: usize> Loggable for &[L; N] {
    fn allocate<T: Loggable>(tag: TagID<T>, builder: impl LogBuilder) {
        for i in 0..N {
            L::allocate(tag, builder.namespace(&i.to_string()));
        }
    }

    fn record<T: Loggable>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
        for i in 0..N {
            self[i].record(tag, &mut logger);
        }
    }
}

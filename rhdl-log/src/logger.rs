use crate::loggable::Loggable;
use crate::tag_id::TagID;

pub trait Logger: Sized {
    type Impl: LoggerImpl;
    fn set_time_in_fs(&mut self, time: u64);
    fn log<L: Loggable>(&mut self, tag: TagID<L>, val: L) {
        val.record(tag, self.get_impl())
    }
    fn get_impl(&mut self) -> &mut Self::Impl;
}

impl<T: LoggerImpl> LoggerImpl for &mut T {
    fn write_bool<L: Loggable>(&mut self, tag: TagID<L>, val: bool) {
        (**self).write_bool(tag, val)
    }
    fn write_bits<L: Loggable>(&mut self, tag: TagID<L>, val: u128) {
        (**self).write_bits(tag, val)
    }
    fn write_string<L: Loggable>(&mut self, tag: TagID<L>, val: &'static str) {
        (**self).write_string(tag, val)
    }
}

pub trait LoggerImpl: Sized {
    fn write_bool<L: Loggable>(&mut self, tag: TagID<L>, val: bool);
    fn write_bits<L: Loggable>(&mut self, tag: TagID<L>, val: u128);
    fn write_string<L: Loggable>(&mut self, tag: TagID<L>, val: &'static str);
}

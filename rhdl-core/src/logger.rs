use crate::{digital::Digital, tag_id::TagID};

pub trait Logger: Sized {
    type Impl: LoggerImpl;
    fn set_time_in_fs(&mut self, time: u64);
    fn log<T: Digital>(&mut self, tag: TagID<T>, val: T) {
        val.record(tag, self.get_impl())
    }
    fn get_impl(&mut self) -> &mut Self::Impl;
}

impl<T: LoggerImpl> LoggerImpl for &mut T {
    fn write_bool<S: Digital>(&mut self, tag: TagID<S>, val: bool) {
        (**self).write_bool(tag, val)
    }
    fn write_bits<S: Digital>(&mut self, tag: TagID<S>, val: u128) {
        (**self).write_bits(tag, val)
    }
    fn write_string<S: Digital>(&mut self, tag: TagID<S>, val: &'static str) {
        (**self).write_string(tag, val)
    }
    fn skip<S: Digital>(&mut self, tag: TagID<S>) {
        (**self).skip(tag)
    }
}

pub trait LoggerImpl: Sized {
    fn write_bool<S: Digital>(&mut self, tag: TagID<S>, val: bool);
    fn write_bits<S: Digital>(&mut self, tag: TagID<S>, val: u128);
    fn write_string<S: Digital>(&mut self, tag: TagID<S>, val: &'static str);
    fn skip<S: Digital>(&mut self, tag: TagID<S>);
}

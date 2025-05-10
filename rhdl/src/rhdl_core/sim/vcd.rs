use std::{io::Write, path::Path};

use sha2::Digest;

use crate::{
    prelude::trace::svg::SvgOptions,
    rhdl_core::{trace::db::TraceDBGuard, trace_init_db, Digital, TimedSample},
};

pub struct Vcd {
    guard: TraceDBGuard,
    time_set: fnv::FnvHashSet<u64>,
}

impl<A> FromIterator<TimedSample<A>> for Vcd
where
    A: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TimedSample<A>>,
        A: Digital,
    {
        let guard = trace_init_db();
        let iter = iter.into_iter();
        let time_set = iter.map(|sample| sample.time).collect();
        Vcd { guard, time_set }
    }
}

impl Vcd {
    pub fn dump<W: Write>(self, writer: W) -> std::io::Result<()> {
        let db = self.guard.take();
        db.dump_vcd(writer, Some(&self.time_set))
    }
    pub fn dump_to_file<P: AsRef<Path>>(self, path: P) -> std::io::Result<String> {
        let mut buf = vec![];
        self.dump(&mut buf)?;
        let hash = sha2::Sha256::digest(&buf);
        std::fs::write(path, &buf)?;
        Ok(format!("{:x}", hash))
    }
    pub fn dump_svg(self, options: &SvgOptions) -> svg::Document {
        let db = self.guard.take();
        let min = self.time_set.iter().min().copied().unwrap_or_default();
        let max = self.time_set.iter().max().copied().unwrap_or_default();
        db.dump_svg(min..=max, options)
    }
}

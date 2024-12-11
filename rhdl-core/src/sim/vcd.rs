use std::{io::Write, path::Path};

use sha2::Digest;

use crate::{trace::db::TraceDBGuard, trace_init_db, Digital, TimedSample};

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
    pub fn dump_to_file(self, path: &Path) -> std::io::Result<String> {
        let mut buf = vec![];
        self.dump(&mut buf)?;
        let hash = sha2::Sha256::digest(&buf);
        std::fs::write(path, &buf)?;
        Ok(format!("{:x}", hash))
    }
}

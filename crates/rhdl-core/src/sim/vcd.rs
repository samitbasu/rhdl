use std::{io::Write, path::Path};

use sha2::Digest;

use crate::{
    Digital,
    trace2::{TraceContainer, session::Session, trace_sample::TraceSample, vcd::VcdFile},
};

pub struct Vcd {
    inner: VcdFile,
}

impl<A> FromIterator<TraceSample<A>> for Vcd
where
    A: Digital,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = TraceSample<A>>,
        A: Digital,
    {
        let mut vcd = VcdFile::default();
        let iter = iter.into_iter();
        for sample in iter {
            vcd.record(&sample)
                .expect("Failed to record sample into VCD");
        }
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
        Ok(format!("{hash:x}"))
    }
    pub fn dump_svg(self, options: &SvgOptions) -> svg::Document {
        let db = self.guard.take();
        let min = self.time_set.iter().min().copied().unwrap_or_default();
        let max = self.time_set.iter().max().copied().unwrap_or_default();
        db.dump_svg(min..=max, options)
    }
}

use std::path::{Path, PathBuf};

use crate::{
    trace::db::{with_trace_db, TraceDBGuard},
    trace_init_db, Digital, TimedSample,
};

pub struct VCDFile<I> {
    _guard: TraceDBGuard,
    time_set: fnv::FnvHashSet<u64>,
    iter: I,
    file_name: PathBuf,
}

pub fn vcd_file<I>(stream: I, file: &Path) -> VCDFile<I> {
    VCDFile {
        _guard: trace_init_db(),
        time_set: fnv::FnvHashSet::default(),
        iter: stream,
        file_name: file.to_path_buf(),
    }
}

impl<T, I> Iterator for VCDFile<I>
where
    T: Digital,
    I: Iterator<Item = TimedSample<T>>,
{
    type Item = TimedSample<T>;

    fn next(&mut self) -> Option<TimedSample<T>> {
        if let Some(sample) = self.iter.next() {
            self.time_set.insert(sample.time);
            Some(sample)
        } else {
            with_trace_db(|db| {
                let fs = std::fs::File::create(&self.file_name).unwrap();
                let buf = std::io::BufWriter::new(fs);
                db.dump_vcd(buf, Some(&self.time_set)).unwrap();
            });
            None
        }
    }
}

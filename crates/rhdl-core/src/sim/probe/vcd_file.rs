use std::path::{Path, PathBuf};

use crate::{
    Digital,
    trace::{TraceContainer, trace_sample::TracedSample, vcd::Vcd},
};

pub struct VCDFile<I> {
    inner: Vcd,
    iter: I,
    file_name: PathBuf,
}

pub fn vcd_file<I>(stream: I, file: &Path) -> VCDFile<I> {
    VCDFile {
        inner: Vcd::default(),
        iter: stream,
        file_name: file.to_path_buf(),
    }
}

impl<T, S, I> Iterator for VCDFile<I>
where
    T: Digital,
    S: Digital,
    I: Iterator<Item = TracedSample<T, S>>,
{
    type Item = TracedSample<T, S>;

    fn next(&mut self) -> Option<TracedSample<T, S>> {
        if let Some(sample) = self.iter.next() {
            self.inner
                .record(&sample)
                .expect("Failed to record sample into VCD");
            Some(sample)
        } else {
            let vcd = std::mem::take(&mut self.inner);
            vcd.dump_to_file(&self.file_name)
                .expect("Failed to write VCD file");
            None
        }
    }
}

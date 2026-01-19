//! Probe to write traced samples to a VCD file without consuming them
use std::path::{Path, PathBuf};

use crate::{
    Digital,
    trace::{
        container::{TraceContainer, vcd::vcd_file::VcdFile},
        trace_sample::TracedSample,
    },
};

/// A probe that writes traced samples to a VCD file without consuming them.
/// See the [book] for an example of its use.
pub struct VcdTap<I> {
    inner: VcdFile,
    iter: I,
    file_name: PathBuf,
}

/// Create a VCD file-writing probe over the supplied stream of traced samples.
pub fn vcd_tap<I>(stream: I, file: &Path) -> VcdTap<I> {
    VcdTap {
        inner: VcdFile::default(),
        iter: stream,
        file_name: file.to_path_buf(),
    }
}

impl<T, S, I> Iterator for VcdTap<I>
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

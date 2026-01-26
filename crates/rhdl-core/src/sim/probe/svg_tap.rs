//! Probe to write traced samples to a Svg file without consuming them
use std::path::{Path, PathBuf};

use crate::{
    Digital,
    trace::{
        container::{
            TraceContainer,
            svg::{options::SvgOptions, svg_file::SvgFile},
        },
        trace_sample::TracedSample,
    },
};

/// A probe that writes traced samples to a Svg file without consuming them.
/// See the [book] for an example of its use.
pub struct SvgTap<I> {
    inner: SvgFile,
    iter: I,
    file_name: PathBuf,
    options: SvgOptions,
}

/// Create a Svg file-writing probe over the supplied stream of traced samples.
pub fn svg_tap<I>(stream: I, file: impl AsRef<Path>, options: SvgOptions) -> SvgTap<I> {
    SvgTap {
        inner: SvgFile::default(),
        iter: stream,
        file_name: file.as_ref().to_path_buf(),
        options,
    }
}

impl<T, S, I> Iterator for SvgTap<I>
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
                .expect("Failed to record sample into Svg");
            Some(sample)
        } else {
            let svg = std::mem::take(&mut self.inner);
            let file = std::fs::File::create(&self.file_name).expect("Failed to create Svg file");
            let mut buffer = std::io::BufWriter::new(file);
            svg.finalize(&self.options, &mut buffer)
                .expect("Failed to write Svg data");
            None
        }
    }
}

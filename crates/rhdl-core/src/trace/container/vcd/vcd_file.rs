//! A VCD file to contain trace samples.

use std::{
    io::Seek,
    io::Write,
    sync::{Arc, RwLock},
};

use tempfile::SpooledTempFile;
use vcd::IdCode;

use crate::{
    BitX, Digital, RHDLError,
    trace::{
        TraceId,
        container::{TraceContainer, vcd::options::VcdOptions},
        meta::TraceMetadata,
        trace_sample::TracedSample,
        trace_tree::TraceTree,
    },
};

/// A VCD trace container that writes trace pages to a VCD file.
/// See the [book] for examples on how to use it.
pub struct VcdFile {
    buffer: std::io::BufWriter<SpooledTempFile>,
    next_id_code: IdCode,
    id_code_map: fnv::FnvHashMap<TraceId, Box<[u8]>>,
    prev_values: fnv::FnvHashMap<TraceId, Box<[BitX]>>,
    db: Option<Arc<RwLock<TraceMetadata>>>,
    last_time: u64,
}

impl Default for VcdFile {
    fn default() -> Self {
        VcdFile::new()
    }
}

impl<T: Digital, S: Digital> FromIterator<TracedSample<T, S>> for VcdFile {
    fn from_iter<I: IntoIterator<Item = TracedSample<T, S>>>(iter: I) -> Self {
        let mut vcd = VcdFile::new();
        for sample in iter {
            vcd.record(&sample)
                .expect("Failed to record sample into VCD");
        }
        vcd
    }
}

impl VcdFile {
    /// Create a new empty VCD trace container.
    pub fn new() -> Self {
        let buffer = std::io::BufWriter::new(SpooledTempFile::new(100 * 1024 * 1024));
        Self {
            buffer,
            next_id_code: IdCode::FIRST,
            id_code_map: fnv::FnvHashMap::default(),
            prev_values: fnv::FnvHashMap::default(),
            db: None,
            last_time: 0,
        }
    }
    fn write_scope(
        &mut self,
        out: &mut impl std::io::Write,
        top: &str,
        tree: &TraceTree,
    ) -> std::io::Result<()> {
        out.write_fmt(format_args!("$scope module {} $end\n", top))?;
        for (name, subtree) in &tree.children {
            self.write_scope(out, name, subtree)?;
        }
        for (name, trace_id) in &tree.signals {
            let name_sanitized = name.replace("::", "__");
            let width = if let Some(db) = self.db.as_ref() {
                let details = db.read().unwrap();
                if let Some(trace_details) = details.get_details(*trace_id) {
                    trace_details.width
                } else {
                    1
                }
            } else {
                1
            };
            let id_code = &self.id_code(*trace_id);
            out.write_fmt(format_args!(
                "$var wire {} {} {} $end\n",
                width,
                std::str::from_utf8(id_code).unwrap(),
                name_sanitized
            ))?;
            self.next_id_code = self.next_id_code.next();
        }
        out.write_all(b"$upscope $end\n")?;
        Ok(())
    }
    fn write_id_code(&mut self, trace_id: TraceId) -> std::io::Result<()> {
        let id_code = self.id_code(trace_id);
        self.buffer.write_all(&id_code)
    }
    fn id_code(&mut self, trace_id: TraceId) -> Box<[u8]> {
        self.id_code_map
            .entry(trace_id)
            .or_insert_with(|| {
                let code = self.next_id_code;
                self.next_id_code = self.next_id_code.next();
                code.to_string().into_bytes().into_boxed_slice()
            })
            .clone()
    }
    /// Finalize the VCD file, writing headers and flushing the buffer.
    pub fn finalize(
        mut self,
        options: &VcdOptions,
        path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<()> {
        let Some(db) = self.db.as_ref() else {
            return Ok(());
        };
        self.buffer.flush()?;
        let out = std::fs::File::create(path.as_ref())?;
        let mut out = std::io::BufWriter::new(out);
        writeln!(out, "$timescale 1 ps $end")?;
        let rtt = db.read().unwrap().rtt();
        let trace_tree = db.read().unwrap().build_trace_tree();
        self.write_scope(&mut out, "top", &trace_tree)?;
        writeln!(out, "$enddefinitions $end")?;
        let mut body = self.buffer.into_inner()?;
        body.seek(std::io::SeekFrom::Start(0))?;
        std::io::copy(&mut body, &mut out)?;
        writeln!(out, "#{}", self.last_time + options.tail_flush_time)?;
        std::fs::write(
            path.as_ref().with_added_extension("rhdl"),
            ron::ser::to_string(&rtt).unwrap(),
        )?;
        Ok(())
    }
    /// Dump the VCD file to the given path, returning the SHA256 hash of the file.
    pub fn dump_to_file(self, path: impl AsRef<std::path::Path>) -> std::io::Result<String> {
        use sha2::Digest;
        self.finalize(&VcdOptions::default(), path.as_ref())?;
        let mut file = std::fs::File::open(path)?;
        let mut hash = sha2::Sha256::default();
        std::io::copy(&mut file, &mut hash)?;
        Ok(format!("{:x}", hash.finalize()))
    }
}

impl TraceContainer for VcdFile {
    fn record<T: Digital, S: Digital>(
        &mut self,
        sample: &TracedSample<T, S>,
    ) -> Result<(), RHDLError> {
        if let Some(page) = sample.page.as_ref() {
            if self.db.is_none() {
                self.db = Some(page.details.clone());
            }
            // Write the time stamp
            self.buffer.write_fmt(format_args!("#{}\n", sample.time))?;
            self.last_time = sample.time;
            let mut sbuf = Vec::new();
            for record in page.records() {
                let value = record.data.bin();
                // Check to see if this value has changed since last time
                let changed = self
                    .prev_values
                    .get(&record.trace_id)
                    .map(|prev| *prev != value)
                    .unwrap_or(true);
                if !changed {
                    continue;
                }
                if value.is_empty() {
                    continue;
                }
                sbuf.clear();
                // Value has changed.  Write it out.  Get the VCD ID code for this trace ID.
                sbuf.push(b'b');
                sbuf.extend(value.iter().rev().map(|v| match v {
                    BitX::Zero => b'0',
                    BitX::One => b'1',
                    BitX::X => b'x',
                }));
                sbuf.push(b' ');
                self.buffer.write_all(&sbuf[..])?;
                self.write_id_code(record.trace_id)?;
                self.buffer.write_all(b"\n")?;
                // Update the previous values
                self.prev_values.insert(record.trace_id, value);
            }
        }
        Ok(())
    }
}

use std::{
    collections::hash_map::Entry,
    hash::{Hash, Hasher},
    io::Write,
};

use smallvec::SmallVec;
use vcd::IdCode;

use crate::{Digital, Kind, NoteKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceBit {
    Zero,
    One,
    X,
    Z,
}

type TimeSeriesHash = u32;

pub type TraceValue = SmallVec<[TraceBit; 8]>;

fn bits_to_vcd(value: &TraceValue, buf: &mut SmallVec<[u8; 64]>) {
    value.iter().for_each(|b| match b {
        TraceBit::Zero => buf.push(b'0'),
        TraceBit::One => buf.push(b'1'),
        TraceBit::X => buf.push(b'x'),
        TraceBit::Z => buf.push(b'z'),
    });
}

struct Cursor {
    next_time: Option<u64>,
    hash: TimeSeriesHash,
    ptr: usize,
    code: IdCode,
    code_as_bytes: Vec<u8>,
}

struct TimeSeries {
    values: Vec<(u64, TraceValue)>,
    kind: Kind,
}

struct TimeSeriesDetails {
    hash: TimeSeriesHash,
    path: Vec<&'static str>,
    key: String,
}

impl TimeSeries {
    fn new(time: u64, value: TraceValue, kind: Kind) -> Self {
        TimeSeries {
            values: vec![(time, value)],
            kind,
        }
    }
    fn push_if_changed(&mut self, time: u64, value: TraceValue) {
        if let Some((_, last_value)) = self.values.last() {
            if last_value == &value {
                return;
            }
        }
        self.values.push((time, value));
    }
    fn cursor<W: Write>(
        &self,
        details: &TimeSeriesDetails,
        name: &str,
        writer: &mut vcd::Writer<W>,
    ) -> Option<Cursor> {
        let name_sanitized = name.replace("::", "__");
        let code = writer
            .add_wire(self.kind.bits() as u32, &name_sanitized)
            .ok()?;
        self.values.first().map(|x| Cursor {
            next_time: Some(x.0),
            hash: details.hash,
            ptr: 0,
            code,
            code_as_bytes: code.to_string().into_bytes(),
        })
    }
    fn advance_cursor(&self, cursor: &mut Cursor) {
        cursor.ptr += 1;
        if let Some((time, _)) = self.values.get(cursor.ptr) {
            cursor.next_time = Some(*time);
        } else {
            cursor.next_time = None;
        }
    }
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        let mut sbuf = SmallVec::<[u8; 64]>::new();
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            sbuf.push(b'b');
            bits_to_vcd(value, &mut sbuf);
            sbuf.push(b' ');
            writer.writer().write_all(&sbuf[..])?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No more values"))
        }
    }
}

#[derive(Default)]
pub struct TraceDB {
    db: fnv::FnvHashMap<TimeSeriesHash, TimeSeries>,
    details: fnv::FnvHashMap<TimeSeriesHash, TimeSeriesDetails>,
    path: Vec<&'static str>,
    time: u64,
}

impl TraceDB {
    fn push_path(&mut self, name: &'static str) {
        self.path.push(name);
    }
    fn pop_path(&mut self) {
        self.path.pop();
    }
    fn key_hash(&self, key: &impl NoteKey) -> TimeSeriesHash {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        hasher.finish() as TimeSeriesHash
    }
    fn trace(&mut self, key: impl NoteKey, value: &impl Digital) {
        let hash = self.key_hash(&key);
        // TODO - placeholder until we have a dedicated trace method on Digital.
        let value_as_trace = value
            .bin()
            .iter()
            .map(|b| match b {
                false => TraceBit::Zero,
                true => TraceBit::One,
            })
            .collect();
        match self.db.entry(hash) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push_if_changed(self.time, value_as_trace)
            }
            Entry::Vacant(entry) => {
                let kind = value.kind();
                eprintln!(
                    "Defining new time series: {path:?} {key} {kind:?}",
                    path = self.path,
                    key = key.as_string(),
                    kind = kind
                );
                let details = TimeSeriesDetails {
                    hash,
                    path: self.path.clone(),
                    key: key.as_string().to_string(),
                };
                self.details.insert(hash, details);
                entry.insert(TimeSeries::new(self.time, value_as_trace, kind));
            }
        }
    }
}

use std::{
    any::Any,
    cell::RefCell,
    collections::{hash_map::Entry, BTreeMap},
    hash::{Hash, Hasher},
    io::Write,
    ops::RangeInclusive,
};

use rhdl_trace_type::{TraceType, RTT};
use smallvec::SmallVec;

use crate::Digital;

use super::{bit::TraceBit, key::TraceKey, vcd::VCDWrite};

type TimeSeriesHash = u32;

// This trait object captures those methods that are needed to walk the time series.
trait TimeSeriesWalk {
    fn cursor(
        &self,
        details: &TimeSeriesDetails,
        name: &str,
        writer: &mut dyn VCDWrite,
        start_time: u64,
    ) -> Option<Cursor>;
    fn advance_cursor(&self, cursor: &mut Cursor);
    fn write_vcd(&self, cursor: &mut Cursor, writer: &mut dyn VCDWrite) -> std::io::Result<()>;
}

struct Cursor {
    next_time: Option<u64>,
    hash: TimeSeriesHash,
    ptr: usize,
    code_as_bytes: Vec<u8>,
}

trait AsAny {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Digital> AsAny for TimeSeries<T> {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct TimeSeries<T: Digital>(Vec<(u64, T)>);

struct TimeSeriesDetails {
    hash: TimeSeriesHash,
    trace_type: TraceType,
    path: Vec<&'static str>,
    key: String,
}

impl<T: Digital> TimeSeries<T> {
    fn new(time: u64, value: T) -> Self {
        let mut values = Vec::with_capacity(1_000_000);
        values.push((time, value));
        TimeSeries(values)
    }
    fn push_if_changed(&mut self, time: u64, value: T) {
        if let Some((_, last_value)) = self.0.last() {
            if last_value == &value {
                return;
            }
        }
        self.0.push((time, value));
    }
}

trait AnyTimeSeries: AsAny + TimeSeriesWalk {}

impl<T: Digital> AnyTimeSeries for TimeSeries<T> {}

impl<T: Digital> TimeSeriesWalk for TimeSeries<T> {
    fn cursor(
        &self,
        details: &TimeSeriesDetails,
        name: &str,
        writer: &mut dyn VCDWrite,
        start_time: u64,
    ) -> Option<Cursor> {
        let name_sanitized = name.replace("::", "__");
        let code = writer
            .add_wire(T::TRACE_BITS as u32, &name_sanitized)
            .ok()?;
        self.0
            .first()
            .map(|x| Cursor {
                next_time: Some(x.0),
                hash: details.hash,
                ptr: 0,
                code_as_bytes: code.to_string().into_bytes(),
            })
            // Fast forward the cursor to the first time in the range
            .and_then(|mut cursor| {
                while let Some((time, _)) = self.0.get(cursor.ptr) {
                    if start_time <= *time {
                        cursor.next_time = Some(*time);
                        return Some(cursor);
                    }
                    cursor.ptr += 1;
                }
                None
            })
    }
    fn advance_cursor(&self, cursor: &mut Cursor) {
        cursor.ptr += 1;
        if let Some((time, _)) = self.0.get(cursor.ptr) {
            cursor.next_time = Some(*time);
        } else {
            cursor.next_time = None;
        }
    }
    fn write_vcd(&self, cursor: &mut Cursor, writer: &mut dyn VCDWrite) -> std::io::Result<()> {
        let mut sbuf = SmallVec::<[u8; 64]>::new();
        if let Some((_time, value)) = self.0.get(cursor.ptr) {
            sbuf.push(b'b');
            sbuf.extend(value.trace().into_iter().rev().map(|v| match v {
                TraceBit::Zero => b'0',
                TraceBit::One => b'1',
                TraceBit::X => b'x',
                TraceBit::Z => b'z',
            }));
            sbuf.push(b' ');
            writer.write_all(&sbuf[..])?;
            writer.write_all(&cursor.code_as_bytes)?;
            writer.write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "No more values",
            ))
        }
    }
}

#[derive(Default)]
pub struct TraceDB {
    db: fnv::FnvHashMap<TimeSeriesHash, Box<dyn AnyTimeSeries>>,
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
    fn key_hash(&self, key: &impl TraceKey) -> TimeSeriesHash {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        hasher.finish() as TimeSeriesHash
    }
    fn trace(&mut self, key: impl TraceKey, value: &impl Digital) {
        let hash = self.key_hash(&key);
        match self.db.entry(hash) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .as_any_mut()
                    .downcast_mut::<TimeSeries<_>>()
                    .unwrap_or_else(|| panic!("Type mismatch for {}", key.as_string()))
                    .push_if_changed(self.time, *value);
            }
            Entry::Vacant(entry) => {
                let details = TimeSeriesDetails {
                    hash,
                    path: self.path.clone(),
                    key: key.as_string().to_string(),
                    trace_type: value.trace_type(),
                };
                self.details.insert(hash, details);
                entry.insert(Box::new(TimeSeries::new(self.time, *value)));
            }
        }
    }
    fn setup_cursor<W: Write>(
        &self,
        name: &str,
        details: &TimeSeriesDetails,
        writer: &mut vcd::Writer<W>,
        start_time: u64,
    ) -> Option<Cursor> {
        self.db
            .get(&details.hash)
            .and_then(|series| series.cursor(details, name, writer, start_time))
    }
    fn write_advance_cursor<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> std::io::Result<()> {
        let series = self.db.get(&cursor.hash).unwrap();
        series.write_vcd(cursor, writer)
    }
    fn advance_cursor(&self, cursor: &mut Cursor) {
        let series = self.db.get(&cursor.hash).unwrap();
        series.advance_cursor(cursor);
    }
    fn setup_cursors<W: Write>(
        &self,
        name: &str,
        scope: &Scope,
        cursors: &mut Vec<Cursor>,
        writer: &mut vcd::Writer<W>,
        start_time: u64,
    ) -> std::io::Result<()> {
        writer.add_module(name)?;
        for (name, hash) in &scope.signals {
            let details = self.details.get(hash).unwrap();
            if let Some(cursor) = self.setup_cursor(name, details, writer, start_time) {
                cursors.push(cursor);
            }
        }
        for (name, child) in &scope.children {
            self.setup_cursors(name, child, cursors, writer, start_time)?;
        }
        writer.upscope()?;
        Ok(())
    }
    fn collect_rtt_info(&self) -> RTT {
        RTT::TraceInfo(
            self.details
                .values()
                .map(|details| {
                    let name = format!(
                        "{}.{}",
                        [&["top"], &details.path[..]].concat().join("."),
                        details.key
                    );
                    let ty = details.trace_type.clone();
                    (name, ty)
                })
                .collect(),
        )
    }
    pub fn dump_vcd<W: Write>(
        &self,
        w: W,
        time_set: Option<&fnv::FnvHashSet<u64>>,
    ) -> std::io::Result<()> {
        let mut writer = vcd::Writer::new(w);
        writer.timescale(1, vcd::TimescaleUnit::PS)?;
        let rtt = self.collect_rtt_info();
        writer.comment(&ron::ser::to_string(&rtt).unwrap())?;
        let root_scope = hierarchical_walk(self.details.iter().map(|(hash, details)| TSItem {
            path: &details.path,
            name: &details.key,
            hash: *hash,
        }));
        let mut cursors = vec![];
        let min_time = time_set.and_then(|x| x.iter().copied().min()).unwrap_or(0);
        self.setup_cursors("top", &root_scope, &mut cursors, &mut writer, min_time)?;
        writer.enddefinitions()?;
        writer.timestamp(0)?;
        let mut current_time = 0;
        let mut keep_running = true;
        while keep_running {
            keep_running = false;
            let mut next_time = !0;
            let mut found_match = true;
            while found_match {
                found_match = false;
                for cursor in &mut cursors {
                    if cursor.next_time == Some(current_time) {
                        if time_set.map(|x| x.contains(&current_time)).unwrap_or(true) {
                            self.write_advance_cursor(cursor, &mut writer)?;
                        } else {
                            self.advance_cursor(cursor);
                        }
                        found_match = true;
                    } else if let Some(time) = cursor.next_time {
                        next_time = next_time.min(time);
                    }
                    if cursor.next_time.is_some() {
                        keep_running = true;
                    }
                }
            }
            if next_time != !0 {
                current_time = next_time;
                if time_set.map(|x| x.contains(&current_time)).unwrap_or(true) {
                    writer.timestamp(current_time)?;
                }
            }
        }
        Ok(())
    }
}

struct TSItem<'a> {
    path: &'a [&'static str],
    name: &'a str,
    hash: TimeSeriesHash,
}

#[derive(Default)]
struct Scope {
    children: BTreeMap<&'static str, Box<Scope>>,
    signals: BTreeMap<String, TimeSeriesHash>,
}

fn hierarchical_walk<'a>(paths: impl Iterator<Item = TSItem<'a>>) -> Scope {
    let mut root = Scope::default();
    for ts_item in paths {
        let mut folder = &mut root;
        for item in ts_item.path {
            if !folder.children.contains_key(item) {
                let new_folder = Box::new(Scope::default());
                folder.children.insert(item, new_folder);
            }
            folder = folder.children.get_mut(item).unwrap();
        }
        folder.signals.insert(ts_item.name.into(), ts_item.hash);
    }
    root
}

thread_local! {
    static DB: RefCell<Option<TraceDB>> = const { RefCell::new(None) };
}
pub struct TraceDBGuard;

impl TraceDBGuard {
    pub fn take(self) -> TraceDB {
        let opt = DB.with(|db| db.borrow_mut().take());
        opt.unwrap_or_default()
    }
}

impl Drop for TraceDBGuard {
    fn drop(&mut self) {
        DB.with(|db| {
            let mut db = db.borrow_mut();
            *db = None;
        });
    }
}

#[must_use]
pub fn trace_init_db() -> TraceDBGuard {
    DB.replace(Some(TraceDB::default()));
    TraceDBGuard {}
}

pub fn with_trace_db<F: FnMut(&TraceDB)>(mut f: F) {
    DB.with(|db| {
        let db = db.borrow();
        if let Some(db) = db.as_ref() {
            f(db);
        }
    })
}

pub fn trace_push_path(name: &'static str) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            db.push_path(name);
        }
    })
}

pub fn trace_pop_path() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            db.pop_path();
        }
    })
}

pub fn trace_time(time: u64) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            db.time = time;
        }
    })
}

pub fn trace(key: impl TraceKey, value: &impl Digital) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        if let Some(db) = db.as_mut() {
            db.trace(key, value)
        }
    })
}

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use rhdl_bits::Bits;

    use crate::{rtt::kind_to_trace, types::kind::Variant, Digital, DiscriminantAlignment, Kind};

    use super::*;

    #[test]
    fn test_vcd_write() {
        let guard = trace_init_db();
        for i in 0..1000 {
            trace_time(i * 1000);
            trace("a", &(i % 2 == 0));
            trace("b", &(i % 2 == 1));
        }
        let mut vcd = vec![];
        let db = guard.take();
        db.dump_vcd(&mut vcd, None).unwrap();
        std::fs::write("test.vcd", vcd).unwrap();
    }

    #[test]
    fn test_vcd_with_enum() {
        #[derive(Copy, Clone, PartialEq, Default)]
        enum Mixed {
            #[default]
            None,
            Bool(bool),
            Tuple(bool, Bits<3>),
            Array([bool; 3]),
            Strct {
                a: bool,
                b: Bits<3>,
            },
        }

        impl Digital for Mixed {
            const BITS: usize = 7;
            fn static_kind() -> Kind {
                Kind::make_enum(
                    "Mixed",
                    vec![
                        Variant {
                            name: "None".to_string(),
                            discriminant: 0,
                            kind: Kind::Empty,
                        },
                        Variant {
                            name: "Bool".to_string(),
                            discriminant: 1,
                            kind: Kind::make_bits(1),
                        },
                        Variant {
                            name: "Tuple".to_string(),
                            discriminant: 2,
                            kind: Kind::make_tuple(vec![Kind::make_bits(1), Kind::make_bits(3)]),
                        },
                        Variant {
                            name: "Array".to_string(),
                            discriminant: 3,
                            kind: Kind::make_array(Kind::make_bits(1), 3),
                        },
                        Variant {
                            name: "Strct".to_string(),
                            discriminant: 4,
                            kind: Kind::make_struct(
                                "Mixed::Strct",
                                vec![
                                    Kind::make_field("a", Kind::make_bits(1)),
                                    Kind::make_field("b", Kind::make_bits(3)),
                                ],
                            ),
                        },
                    ],
                    Kind::make_discriminant_layout(
                        3,
                        DiscriminantAlignment::Lsb,
                        crate::types::kind::DiscriminantType::Unsigned,
                    ),
                )
            }
            fn static_trace_type() -> rhdl_trace_type::TraceType {
                kind_to_trace(&Self::static_kind())
            }
            fn bin(self) -> Vec<bool> {
                let raw = match self {
                    Self::None => rhdl_bits::bits::<3>(0).to_bools(),
                    Self::Bool(b) => {
                        let mut v = rhdl_bits::bits::<3>(1).to_bools();
                        v.extend(b.bin());
                        v
                    }
                    Self::Tuple(b, c) => {
                        let mut v = rhdl_bits::bits::<3>(2).to_bools();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v
                    }
                    Self::Array([b, c, d]) => {
                        let mut v = rhdl_bits::bits::<3>(3).to_bools();
                        v.extend(b.bin());
                        v.extend(c.bin());
                        v.extend(d.bin());
                        v
                    }
                    Self::Strct { a, b } => {
                        let mut v = rhdl_bits::bits::<3>(4).to_bools();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                };
                if raw.len() < self.kind().bits() {
                    let missing = self.kind().bits() - raw.len();
                    raw.into_iter().chain(repeat(false).take(missing)).collect()
                } else {
                    raw
                }
            }
            fn init() -> Self {
                <Self as Default>::default()
            }
        }

        assert_eq!(Mixed::None.kind().bits(), Mixed::BITS);

        let guard = trace_init_db();
        trace_time(0);
        trace("a", &Mixed::None);
        trace_time(100);
        trace("a", &Mixed::Array([true, false, true]));
        trace_time(200);
        trace(
            "a",
            &Mixed::Strct {
                a: true,
                b: rhdl_bits::bits(5),
            },
        );
        trace_time(300);
        trace("a", &Mixed::Bool(false));
        trace_time(400);
        trace("a", &Mixed::Tuple(true, rhdl_bits::bits(3)));
        trace_time(500);

        let mut vcd = vec![];
        let db = guard.take();
        db.dump_vcd(&mut vcd, None).unwrap();
        std::fs::write("test_enum.vcd", vcd).unwrap();
    }

    #[test]
    fn test_vcd_with_nested_paths() {
        let guard = trace_init_db();
        for i in 0..10 {
            trace_time(i * 1000);
            trace_push_path("fn1");
            trace_push_path("fn2");
            trace("a", &true);
            trace_pop_path();
            trace("a", &rhdl_bits::bits::<6>(i as u128));
            trace_pop_path();
        }
        let mut vcd = vec![];
        let db = guard.take();
        db.dump_vcd(&mut vcd, None).unwrap();
        std::fs::write("test_nested_paths.vcd", vcd).unwrap();
    }
}

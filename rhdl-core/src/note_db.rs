//! The note database is a log designed around time series of data values (like an oscilloscope). It can be exported to a VCD file.
//!
//! rhdl uses a global note database singleton to manage all the logged notes. You can access the singleton by using the functions provided by this module. The term _note_ refers to the value for a specific variable at a specific timestamp.
//!
//! Use [`note`] to log values to the note database. After you are done, call [`note_take_vcd`] to dump the note database to a [Value-Change Dump (VCD)](https://en.wikipedia.org/wiki/Value_change_dump) file and reset it.
//!
//! # Examples
//!
//! Dump the simulation of a synchronous module to a VCD file:
//!
//! ```
//! use rhdl::{bits::bits, synchronous::{simulate, OneShot}};
//! use rhdl_core::{note_reset_db, note_take_vcd};
//!
//! // Simulate device
//! let inputs = vec![false, true, false, false, false, false, false, false].into_iter();
//! let dut = OneShot::<26> { duration: bits(3) };
//! simulate(dut, inputs).count();
//!
//! // Write notes to VCD file
//! let mut vcd_file = std::fs::File::create("oneshot.vcd").unwrap();
//! note_take_vcd(&[], &mut vcd_file).unwrap();
//! ```
//!
//! Use the notetaking functions outside of a simulation:
//!
//! ```
//! use rhdl_core::{note, note_reset_db, note_time, note_take_vcd};
//! // Reset db
//! note_reset_db();
//!
//! // Note some values
//! note("a", false);
//! note("b", 42);
//! note_time(1000);
//! note("a", true);
//! note("b", 47);
//!
//! // Dump the note database to a VCD file
//! let mut vcd_file = std::fs::File::create("dump.vcd").unwrap();
//! note_take_vcd(&[], vcd_file).unwrap();
//! ```
//!
//! Usage inside the kernel of a synchronous module:
//!
//! ```
//! use rhdl::{kernel, Digital};
//! use rhdl_core::{note, Synchronous};
//!
//! /// Adds the input to the sum of the previous inputs every clock cycle
//! #[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
//! struct SumAccumulator {}
//!
//! impl Synchronous for SumAccumulator {
//!     type Input = u8;
//!     type Output = u8;
//!     type State = u8;
//!     type Update = sum_accumulator_update;
//!
//!     const INITIAL_STATE: Self::State = 0;
//!     const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
//!         sum_accumulator_update;
//! }
//!
//! #[kernel]
//! pub fn sum_accumulator_update(
//!     _params: SumAccumulator,
//!     state: u8,
//!     input: u8,
//! ) -> (u8, u8) {
//!     // Take notes in the update function
//!     note("input", input);
//!     note("state", state);
//!     let output = input + state;
//!     note("output", output);
//!     return (output, output);
//! }
//!
//! #[cfg(test)]
//! mod tests {
//!     use rhdl::synchronous::simulate;
//!     use rhdl_core::note_take_vcd;
//!
//!     use super::SumAccumulator;
//!     #[test]
//!     fn test_start_pulse_simulation() {
//!         let inputs = vec![4, 6, 8, 12, 3].into_iter();
//!         let dut = SumAccumulator {};
//!         simulate(dut, inputs).count();
//!
//!         let mut vcd_file = std::fs::File::create("sum_accumulator.vcd").unwrap();
//!         note_take_vcd(&[], &mut vcd_file).unwrap();
//!     }
//! }
//! ```
use crate::types::note::Notable;
use crate::{ClockDetails, NoteKey, NoteWriter};
use anyhow::bail;
use std::hash::Hash;
use std::{cell::RefCell, hash::Hasher, io::Write};
use vcd::IdCode;

/// A timestamp in picoseconds since the start of the simulation.
type Time = u64;

/// A time series of values at different times.
struct TimeSeries<T> {
    /// A list of time and value pairs.
    values: Vec<(Time, T)>,
    /// The width of the value in bits.
    width: u8,
}

impl<T> TimeSeries<T> {
    /// Create a new time series with a single value.
    fn new(time: Time, value: T, width: u8) -> Self {
        Self {
            values: vec![(time, value)],
            width,
        }
    }
    fn cursor<W: Write>(
        &self,
        details: &TimeSeriesDetails,
        name: &str,
        writer: &mut vcd::Writer<W>,
    ) -> Option<Cursor> {
        let name_sanitized = name.replace("::", "__");
        let code = writer.add_wire(self.width as u32, &name_sanitized).ok()?;
        self.values.first().map(|x| Cursor {
            kind: details.kind,
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
}

impl TimeSeries<bool> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            writer
                .writer()
                .write_all(if *value { b"1" } else { b"0" })?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl TimeSeries<u128> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        let mut sbuf = [0_u8; 256];
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            sbuf[0] = b'b';
            bits_to_vcd(*value, self.width as usize, &mut sbuf[1..]);
            sbuf[self.width as usize + 1] = b' ';
            writer
                .writer()
                .write_all(&sbuf[0..(self.width as usize + 2)])?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl TimeSeries<i128> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        let mut sbuf = [0_u8; 256];
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            sbuf[0] = b'b';
            bits_to_vcd(*value as u128, self.width as usize, &mut sbuf[1..]);
            sbuf[self.width as usize + 1] = b' ';
            writer
                .writer()
                .write_all(&sbuf[0..(self.width as usize + 2)])?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl TimeSeries<&'static str> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            writer.change_string(cursor.code, value)?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Tristate {
    value: u128,
    mask: u128,
}

impl TimeSeries<Tristate> {
    fn write_vcd<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        let mut sbuf = [0_u8; 256];
        if let Some((_time, value)) = self.values.get(cursor.ptr) {
            sbuf[0] = b'b';
            tristate_to_vcd(value.value, value.mask, self.width as usize, &mut sbuf[1..]);
            sbuf[self.width as usize + 1] = b' ';
            writer
                .writer()
                .write_all(&sbuf[0..(self.width as usize + 2)])?;
            writer.writer().write_all(&cursor.code_as_bytes)?;
            writer.writer().write_all(b"\n")?;
            self.advance_cursor(cursor);
            Ok(())
        } else {
            bail!("No more values")
        }
    }
}

impl<T: PartialEq> TimeSeries<T> {
    fn push(&mut self, time: Time, value: T, width: u8) {
        if let Some((_last_time, last_value)) = self.values.last() {
            if *last_value == value {
                return;
            }
        }
        self.values.push((time, value));
        assert_eq!(self.width, width);
    }
}

type TimeSeriesHash = u32;

struct TimeSeriesDetails {
    kind: TimeSeriesKind,
    hash: TimeSeriesHash,
}

fn tristate_to_vcd(x: u128, mask: u128, width: usize, buffer: &mut [u8]) {
    (0..width).for_each(|i| {
        buffer[i] = if mask & (1 << (width - 1 - i)) != 0 {
            if x & (1 << (width - 1 - i)) != 0 {
                b'1'
            } else {
                b'0'
            }
        } else {
            b'z'
        };
    })
}

fn bits_to_vcd(x: u128, width: usize, buffer: &mut [u8]) {
    (0..width).for_each(|i| {
        buffer[i] = if x & (1 << (width - 1 - i)) != 0 {
            b'1'
        } else {
            b'0'
        };
    })
}

/// A log for hardware designs that is designed around time series of data values (like an oscilloscope) than a text journal of log entries.
///
/// It keeps track of the module that is currently being noted, the current time, and the values of all time series.
///
/// rhdl uses a global note database singleton to store all the notes. You can access this singleton using the functions exported by [`crate::note_db``].
#[derive(Default)]
struct NoteDB {
    /// The names for each time series of values.
    details: fnv::FnvHashMap<String, TimeSeriesDetails>,
    /// The picoseconds since start of the simulation.
    time: Time,
    /// The current path in the design hierarchy.
    ///
    /// The current value will be prepended to all noted keys. Can be adjusted with [`push_path`] and [`pop_path`]. The main idea is to push the current module name before noting values in that module, so that the VCD dump will have the full path to the signal.
    path: Vec<&'static str>,
    db_bool: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<bool>>,
    db_bits: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<u128>>,
    db_signed: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<i128>>,
    db_string: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<&'static str>>,
    db_tristate: fnv::FnvHashMap<TimeSeriesHash, TimeSeries<Tristate>>,
}

struct Cursor {
    next_time: Option<Time>,
    hash: TimeSeriesHash,
    kind: TimeSeriesKind,
    ptr: usize,
    code: IdCode,
    code_as_bytes: Vec<u8>,
}

/// Represents the possible value types of a time series.
#[derive(Copy, Clone, Debug)]
enum TimeSeriesKind {
    Bool,
    Bits,
    Signed,
    String,
    Tristate,
}

/// Implementation for writing notes to the [`NoteDB`].
impl NoteWriter for NoteDB {
    fn write_bool(&mut self, key: impl NoteKey, value: bool) {
        self.note_bool(key, value);
    }

    fn write_bits(&mut self, key: impl NoteKey, value: u128, len: u8) {
        self.note_u128(key, value, len);
    }

    fn write_signed(&mut self, key: impl NoteKey, value: i128, len: u8) {
        self.note_i128(key, value, len);
    }

    fn write_string(&mut self, key: impl NoteKey, value: &'static str) {
        self.note_string(key, value);
    }

    fn write_tristate(&mut self, key: impl NoteKey, value: u128, mask: u128, size: u8) {
        self.note_tristate(key, value, mask, size);
    }
}

impl NoteDB {
    /// See [`note_push_path`].
    fn push_path(&mut self, name: &'static str) {
        self.path.push(name);
    }
    /// See [`note_pop_path`].
    fn pop_path(&mut self) {
        self.path.pop();
    }
    fn note_bool(&mut self, key: impl NoteKey, value: bool) {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as TimeSeriesHash;
        if let Some(values) = self.db_bool.get_mut(&key_hash) {
            values.push(self.time, value, 1);
        } else {
            self.details.insert(
                key.as_string().to_string(),
                TimeSeriesDetails {
                    kind: TimeSeriesKind::Bool,
                    hash: key_hash,
                },
            );
            self.db_bool
                .insert(key_hash, TimeSeries::new(self.time, value, 1));
        }
    }
    fn note_u128(&mut self, key: impl NoteKey, value: u128, width: u8) {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as TimeSeriesHash;
        if let Some(values) = self.db_bits.get_mut(&key_hash) {
            values.push(self.time, value, width);
        } else {
            self.details.insert(
                key.as_string().to_string(),
                TimeSeriesDetails {
                    kind: TimeSeriesKind::Bits,
                    hash: key_hash,
                },
            );
            self.db_bits
                .insert(key_hash, TimeSeries::new(self.time, value, width));
        }
    }
    fn note_i128(&mut self, key: impl NoteKey, value: i128, width: u8) {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as TimeSeriesHash;
        if let Some(values) = self.db_signed.get_mut(&key_hash) {
            values.push(self.time, value, width);
        } else {
            self.details.insert(
                key.as_string().to_string(),
                TimeSeriesDetails {
                    kind: TimeSeriesKind::Signed,
                    hash: key_hash,
                },
            );
            self.db_signed
                .insert(key_hash, TimeSeries::new(self.time, value, width));
        }
    }
    fn note_string(&mut self, key: impl NoteKey, value: &'static str) {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as TimeSeriesHash;
        if let Some(values) = self.db_string.get_mut(&key_hash) {
            values.push(self.time, value, 0);
        } else {
            self.details.insert(
                key.as_string().to_string(),
                TimeSeriesDetails {
                    kind: TimeSeriesKind::String,
                    hash: key_hash,
                },
            );
            self.db_string
                .insert(key_hash, TimeSeries::new(self.time, value, 0));
        }
    }
    fn note_tristate(&mut self, key: impl NoteKey, value: u128, mask: u128, width: u8) {
        let mut hasher = fnv::FnvHasher::default();
        let key = (&self.path[..], key);
        key.hash(&mut hasher);
        let key_hash = hasher.finish() as TimeSeriesHash;
        if let Some(values) = self.db_tristate.get_mut(&key_hash) {
            values.push(self.time, Tristate { value, mask }, width);
        } else {
            self.details.insert(
                key.as_string().to_string(),
                TimeSeriesDetails {
                    kind: TimeSeriesKind::Tristate,
                    hash: key_hash,
                },
            );
            self.db_tristate.insert(
                key_hash,
                TimeSeries::new(self.time, Tristate { value, mask }, width),
            );
        }
    }

    fn setup_cursor<W: Write>(
        &self,
        name: &str,
        details: &TimeSeriesDetails,
        writer: &mut vcd::Writer<W>,
    ) -> Option<Cursor> {
        match details.kind {
            TimeSeriesKind::Bits => self
                .db_bits
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::Bool => self
                .db_bool
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::Signed => self
                .db_signed
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::String => self
                .db_string
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
            TimeSeriesKind::Tristate => self
                .db_tristate
                .get(&details.hash)
                .and_then(|series| series.cursor(details, name, writer)),
        }
    }
    fn write_advance_cursor<W: Write>(
        &self,
        cursor: &mut Cursor,
        writer: &mut vcd::Writer<W>,
    ) -> anyhow::Result<()> {
        match cursor.kind {
            TimeSeriesKind::Bits => self
                .db_bits
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::Bool => self
                .db_bool
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::Signed => self
                .db_signed
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::String => self
                .db_string
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
            TimeSeriesKind::Tristate => self
                .db_tristate
                .get(&cursor.hash)
                .unwrap()
                .write_vcd(cursor, writer),
        }
    }

    /// See [`note_take_vcd`].
    fn dump_vcd<W: Write>(&self, clocks: &[ClockDetails], w: W) -> anyhow::Result<()> {
        let mut writer = vcd::Writer::new(w);
        writer.timescale(1, vcd::TimescaleUnit::PS)?;
        writer.add_module("top")?;
        let clocks = clocks
            .iter()
            .map(|c| {
                (
                    c,
                    writer
                        .add_wire(1, &c.name)
                        .unwrap()
                        .to_string()
                        .into_bytes(),
                )
            })
            .collect::<Vec<_>>();
        let mut cursors: Vec<Cursor> = self
            .details
            .iter()
            .filter_map(|(name, details)| self.setup_cursor(name, details, &mut writer))
            .collect();
        writer.upscope()?;
        writer.enddefinitions()?;
        writer.timestamp(0)?;
        let mut current_time = 0;
        let mut keep_running = true;
        while keep_running {
            keep_running = false;
            let mut next_time = !0;
            for (clock, code) in &clocks {
                if clock.pos_edge_at(current_time) {
                    writer.writer().write_all(b"1")?;
                    writer.writer().write_all(code)?;
                    writer.writer().write_all(b"\n")?;
                } else if clock.neg_edge_at(current_time) {
                    writer.writer().write_all(b"0")?;
                    writer.writer().write_all(code)?;
                    writer.writer().write_all(b"\n")?;
                }
                next_time = next_time.min(clock.next_edge_after(current_time));
            }
            let mut found_match = true;
            while found_match {
                found_match = false;
                for cursor in &mut cursors {
                    if cursor.next_time == Some(current_time) {
                        self.write_advance_cursor(cursor, &mut writer)?;
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
                writer.timestamp(current_time)?;
            }
        }
        Ok(())
    }
}

thread_local! {
    /// The note database singleton.
    ///
    /// It will get initialized when you first use any of the [`note`] functions.
    ///
    /// Use the [`note_take_vcd`] function to get the database and dump it to a VCD file.
    static DB: RefCell<NoteDB> = RefCell::new(NoteDB::default());
}

/// Reset the note database singleton to an empty state.
///
/// # Examples
///
/// Reset the note database singleton:
///
/// ```
/// use rhdl_core::{note_reset_db, note};
///
/// // Note some values
/// note("a", false);
/// note("b", 6);
///
/// // Reset the note database
/// note_reset_db();
///
/// // The note database is now empty again
/// ```
///
/// See [`note_db`](crate::note_db#examples) for more examples.
pub fn note_reset_db() {
    DB.take();
}

/// Add a path to path stack in the design hierarchy.
///
/// The current path stack will be prepended to all noted keys.
///
/// If you are implementing [`crate::Circuit`] yourself, this should be called in the [`crate::Circuit::sim`] method, before calling sim on a child circuit. Remember to call [`note_pop_path`] to remove the path from the stack after simulating the child circuits.
pub fn note_push_path(name: &'static str) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        db.push_path(name);
    });
}

/// Remove a path from the path stack in the design hierarchy.
///
/// See [`note_push_path`] for more information.
pub fn note_pop_path() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        db.pop_path();
    });
}

/// Set the current time.
///
/// The time that will be used for all notes written after this call.
pub fn note_time(time: Time) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        db.time = time;
    });
}

/// Write a value for the key at the current time to the note database singleton.
///
/// The current path prefix (See [`note_push_path`], and [`note_pop_path`]) will be prepended to the key.
///
/// # Examples
///
/// Note a value for `input` to the note database singleton
///
/// ```
/// use rhdl_core::{note};
///
/// note("input", 5);
/// ```
///
/// Logging in a kernel:
///
/// ```
/// use rhdl::kernel;
/// use rhdl_core::note;
///
/// #[kernel]
/// fn adder(a: u8, b: u8) -> u8 {
///     note("a", a);
///     note("b", b);
///     return a + b;
/// }
/// ```
///
/// See [`note_db`](crate::note_db#examples) for more examples.
pub fn note(key: impl NoteKey, value: impl Notable) {
    DB.with(|db| {
        let db: &mut NoteDB = &mut db.borrow_mut();
        value.note(key, db);
    });
}

/// Take and dump the note database singleton to a VCD file.
///
/// This will dump the note database to a VCD file and replace it with a new empty note database. The VCD dump will contain all the values that have been noted since the last [`note_take_vcd`] or [`note_reset_db`].
///
/// The `clocks` parameter can be used to add additional [`ClockDetails`] for reference.
/// # Examples
///
/// Dump the current note database to a VCD file:
///
/// ```
/// use rhdl_core::{note_take_vcd};
///
/// let mut vcd_file = std::fs::File::create("dump.vcd").unwrap();
/// note_take_vcd(&[], vcd_file).unwrap();
/// ```
///
/// Dump VCD into a buffer:
///
/// ```
/// use rhdl_core::{note_take_vcd};
///
/// let mut vcd_content = vec![];
/// note_take_vcd(&[], &mut vcd_content).unwrap();
/// ```
///
/// After calling `note_take_vcd`, the note database will be empty:
///
/// ```
/// use rhdl_core::{note, note_reset_db, note_time, note_take_vcd};
///
/// // Note some values
/// note("a", false);
/// note("b", 5);
///
/// // This will create a VCD file with the notes
/// let mut vcd_file = std::fs::File::create("dump.vcd").unwrap();
/// note_take_vcd(&[], vcd_file).unwrap();
///
/// // This will now create an empty VCD file
/// let mut vcd_file = std::fs::File::create("dump.vcd").unwrap();
/// note_take_vcd(&[], vcd_file).unwrap();
/// ```
///
/// See [`note_db`](crate::note_db#examples) for more examples.
pub fn note_take_vcd<W: Write>(clocks: &[ClockDetails], writer: W) -> anyhow::Result<()> {
    let db = DB.take();
    db.dump_vcd(clocks, writer)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use rhdl_bits::Bits;

    use crate::{types::kind::Variant, Digital, DiscriminantAlignment, Kind};

    use super::*;

    #[test]
    fn test_vcd_write() {
        note_reset_db();
        for i in 0..1000 {
            note_time(i * 1000);
            note("a", i % 2 == 0);
            note("b", i % 2 == 1);
        }
        let mut vcd = vec![];
        let clock = ClockDetails::new("clk", 5, 0, false);
        note_take_vcd(&[clock], &mut vcd).unwrap();
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
        }

        impl Notable for Mixed {
            fn note(&self, key: impl NoteKey, mut writer: impl NoteWriter) {
                match self {
                    Self::None => {
                        writer.write_string(key, stringify!(None));
                    }
                    Self::Bool(b) => {
                        writer.write_string(key, stringify!(Bool));
                        Notable::note(b, (key, 0), &mut writer);
                    }
                    Self::Tuple(b, c) => {
                        writer.write_string(key, stringify!(Tuple));
                        b.note((key, "b"), &mut writer);
                        c.note((key, "c"), &mut writer);
                    }
                    Self::Array([b, c, d]) => {
                        writer.write_string(key, stringify!(Array));
                        b.note((key, 0), &mut writer);
                        c.note((key, 1), &mut writer);
                        d.note((key, 2), &mut writer);
                    }
                    Self::Strct { a, b } => {
                        writer.write_string(key, stringify!(Strct));
                        a.note((key, "a"), &mut writer);
                        b.note((key, "b"), &mut writer);
                    }
                }
            }
        }

        note_reset_db();
        note_time(0);
        note("a", Mixed::None);
        note_time(100);
        note("a", Mixed::Array([true, false, true]));
        note_time(200);
        note(
            "a",
            Mixed::Strct {
                a: true,
                b: rhdl_bits::bits(5),
            },
        );
        note_time(300);
        note("a", Mixed::Bool(false));
        note_time(400);
        note("a", Mixed::Tuple(true, rhdl_bits::bits(3)));
        note_time(500);

        let clock = ClockDetails::new("clk", 100, 0, false);
        let mut vcd = vec![];
        note_take_vcd(&[clock], &mut vcd).unwrap();
        std::fs::write("test_enum.vcd", vcd).unwrap();
    }

    #[test]
    fn test_vcd_with_nested_paths() {
        note_reset_db();
        for i in 0..10 {
            note_time(i * 1000);
            note_push_path("fn1");
            note_push_path("fn2");
            note("a", true);
            note_pop_path();
            note("a", rhdl_bits::bits::<6>(i as u128));
            note_pop_path();
        }
        let mut vcd = vec![];
        let clock = ClockDetails::new("clk", 500, 0, false);
        note_take_vcd(&[clock], &mut vcd).unwrap();
        std::fs::write("test_nested_paths.vcd", vcd).unwrap();
    }
}

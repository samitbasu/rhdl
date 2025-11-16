//! VCD Writer Trait Object Interface
//!
//! This module defines a Trait object that captures the interface
//! to a VCD writer.  This allows us to abstract over different
//! implementations of VCD writers, as long as they implement the
//! required methods.
use std::io::Write;

use vcd::IdCode;

/// This Trait object captures the interface back to the VCD writer.
pub trait VCDWrite {
    /// Set the timescale for the VCD file
    fn timescale(&mut self, magnitude: u32, unit: vcd::TimescaleUnit) -> std::io::Result<()>;
    /// Add a module to the VCD file
    fn add_module(&mut self, name: &str) -> std::io::Result<()>;
    /// Move up one scope in the VCD file
    fn upscope(&mut self) -> std::io::Result<()>;
    /// End the definitions section of the VCD file
    fn enddefinitions(&mut self) -> std::io::Result<()>;
    /// Set the current timestamp in the VCD file
    fn timestamp(&mut self, time: u64) -> std::io::Result<()>;
    /// Add a wire to the VCD file
    fn add_wire(&mut self, width: u32, name: &str) -> std::io::Result<IdCode>;
    /// Write raw bytes to the VCD file
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()>;
}

impl<W: Write> VCDWrite for vcd::Writer<W> {
    fn timescale(&mut self, magnitude: u32, unit: vcd::TimescaleUnit) -> std::io::Result<()> {
        self.timescale(magnitude, unit)
    }
    fn add_module(&mut self, name: &str) -> std::io::Result<()> {
        self.add_module(name)
    }
    fn upscope(&mut self) -> std::io::Result<()> {
        self.upscope()
    }
    fn enddefinitions(&mut self) -> std::io::Result<()> {
        self.enddefinitions()
    }
    fn timestamp(&mut self, time: u64) -> std::io::Result<()> {
        self.timestamp(time)
    }
    fn add_wire(&mut self, width: u32, name: &str) -> std::io::Result<IdCode> {
        self.add_wire(width, name)
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.writer().write_all(buf)
    }
}

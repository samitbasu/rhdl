use std::io::Write;

use vcd::IdCode;

// This Trait object captures the interface back to the VCD writer.
pub trait VCDWrite {
    fn timescale(&mut self, magnitude: u32, unit: vcd::TimescaleUnit) -> std::io::Result<()>;
    fn add_module(&mut self, name: &str) -> std::io::Result<()>;
    fn upscope(&mut self) -> std::io::Result<()>;
    fn enddefinitions(&mut self) -> std::io::Result<()>;
    fn timestamp(&mut self, time: u64) -> std::io::Result<()>;
    fn add_wire(&mut self, width: u32, name: &str) -> std::io::Result<IdCode>;
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

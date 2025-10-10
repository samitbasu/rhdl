//! Some helper utilites to use the `icestorm` toolchain

use rhdl_core::{Circuit, circuit::fixture::Fixture};

use crate::{
    nextpnr_ice40::{NextpnrIce40Output, TimingInfo},
    yosys::YosysOutput,
};
pub struct IceStorm {
    part: String,
    package: String,
    directory: camino::Utf8PathBuf,
}

impl IceStorm {
    pub fn new(part: &str, package: &str, directory: impl Into<camino::Utf8PathBuf>) -> Self {
        Self {
            part: part.to_string(),
            package: package.to_string(),
            directory: directory.into(),
        }
    }
    pub fn clean(&self) -> miette::Result<&Self> {
        std::fs::remove_dir_all(&self.directory)
            .or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .map_err(|e| miette::miette!(e))?;
        std::fs::create_dir_all(&self.directory).map_err(|e| miette::miette!(e))?;
        Ok(self)
    }
    pub fn synth(&self, fixture: Fixture<impl Circuit>) -> miette::Result<YosysOutput> {
        crate::yosys::synth_ice40(fixture.module()?, fixture.name(), &self.directory)
    }
    pub fn pnr(&self, pcf: Option<&str>) -> miette::Result<NextpnrIce40Output> {
        crate::nextpnr_ice40::place_and_route(&self.part, &self.package, pcf, &self.directory)
    }
    pub fn pack(&self) -> miette::Result<()> {
        crate::icepack::run(&self.directory)
    }
    pub fn flash(&self) -> miette::Result<()> {
        crate::openfpgaloader::ice40_generic(&self.directory)
    }
    pub fn build_and_flash<T: Circuit>(
        &self,
        fixture: Fixture<T>,
        pcf: &str,
    ) -> miette::Result<()> {
        self.synth(fixture)?;
        self.pnr(Some(pcf))?;
        self.pack()?;
        self.flash()?;
        Ok(())
    }
    pub fn time(&self, circuit: impl Circuit) -> miette::Result<TimingInfo> {
        let mut fixture = Fixture::new("top", circuit);
        fixture.pass_through_input(
            "inputs",
            &rhdl_core::types::path::Path::default().signal_value(),
        )?;
        fixture.pass_through_output(
            "outputs",
            &rhdl_core::types::path::Path::default().signal_value(),
        )?;
        self.synth(fixture)?;
        let pnr_output = self.pnr(None)?;
        pnr_output.extract_timing()
    }
}

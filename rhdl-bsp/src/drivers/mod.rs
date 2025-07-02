
use rhdl::{
    core::{CircuitIO, RHDLError, Timed},
    prelude::{bit_range, sub_trace_type, Digital, ExportError, MountPoint, Path},
    rtt::TraceType,
};

pub mod lattice;
pub mod xilinx;

pub fn get_untyped_input<T: CircuitIO>(path: &Path, width: usize) -> Result<MountPoint, RHDLError> {
    let (bits, _sub) = bit_range(<T::I as Timed>::static_kind(), path)?;
    if bits.len() != width {
        return Err(RHDLError::ExportError(
            ExportError::SignalWidthMismatchInput {
                expected: width,
                actual: bits.len(),
                path: path.clone(),
            },
        ));
    }
    Ok(MountPoint::Input(bits))
}

pub fn get_untyped_output<T: CircuitIO>(
    path: &Path,
    width: usize,
) -> Result<MountPoint, RHDLError> {
    let (bits, _sub) = bit_range(<T::O as Timed>::static_kind(), path)?;
    if bits.len() != width {
        return Err(RHDLError::ExportError(
            ExportError::SignalWidthMismatchOutput {
                expected: width,
                actual: bits.len(),
                path: path.clone(),
            },
        ));
    }
    Ok(MountPoint::Output(bits))
}

pub fn get_clock_input<T: CircuitIO>(path: &Path) -> Result<MountPoint, RHDLError> {
    let trace_type = <T::I as Digital>::static_trace_type();
    let target_trace = sub_trace_type(trace_type, path)?;
    if target_trace != TraceType::Clock {
        return Err(RHDLError::ExportError(ExportError::NotAClockInput(
            path.clone(),
        )));
    }
    let (bits, sub) = bit_range(<T::I as Timed>::static_kind(), path)?;
    if bits.len() != 1 || sub.is_signal() {
        return Err(RHDLError::ExportError(ExportError::NotAClockInput(
            path.clone(),
        )));
    }
    Ok(MountPoint::Input(bits))
}

pub fn get_clock_output<T: CircuitIO>(path: &Path) -> Result<MountPoint, RHDLError> {
    let trace_type = <T::O as Digital>::static_trace_type();
    let target_trace = sub_trace_type(trace_type, path)?;
    if target_trace != TraceType::Clock {
        return Err(RHDLError::ExportError(ExportError::NotAClockOutput(
            path.clone(),
        )));
    }
    let (bits, sub) = bit_range(<T::O as Timed>::static_kind(), path)?;
    if bits.len() != 1 || sub.is_signal() {
        return Err(RHDLError::ExportError(ExportError::NotAClockOutput(
            path.clone(),
        )));
    }
    Ok(MountPoint::Input(bits))
}

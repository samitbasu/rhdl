use rhdl::{
    core::{types::path::sub_kind, CircuitIO, RHDLError},
    prelude::*,
    rtt::TraceType,
};

pub mod lattice;
pub mod xilinx;

pub fn get_untyped_input<T: CircuitIO>(path: &Path, width: usize) -> Result<MountPoint, RHDLError> {
    let (bits, _sub) = bit_range(<T::I as Digital>::static_kind(), path)?;
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
    let (bits, _sub) = bit_range(<T::O as Digital>::static_kind(), path)?;
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
    let trace_type = <T::I as Digital>::static_kind();
    let target_trace = sub_kind(trace_type, path)?;
    if target_trace != Kind::Clock {
        return Err(RHDLError::ExportError(ExportError::NotAClockInput {
            path: path.clone(),
            kind: <T::I as Digital>::static_kind(),
        }));
    }
    let (bits, sub) = bit_range(<T::I as Digital>::static_kind(), path)?;
    if bits.len() != 1 || sub.is_signal() {
        return Err(RHDLError::ExportError(ExportError::NotAClockInput {
            path: path.clone(),
            kind: <T::I as Digital>::static_kind(),
        }));
    }
    Ok(MountPoint::Input(bits))
}

pub fn get_clock_output<T: CircuitIO>(path: &Path) -> Result<MountPoint, RHDLError> {
    let trace_type = <T::O as Digital>::static_kind();
    let target_trace = sub_kind(trace_type, path)?;
    if target_trace != Kind::Clock {
        return Err(RHDLError::ExportError(ExportError::NotAClockOutput {
            path: path.clone(),
            kind: <T::O as Digital>::static_kind(),
        }));
    }
    let (bits, sub) = bit_range(<T::O as Digital>::static_kind(), path)?;
    if bits.len() != 1 || sub.is_signal() {
        return Err(RHDLError::ExportError(ExportError::NotAClockOutput {
            path: path.clone(),
            kind: <T::O as Digital>::static_kind(),
        }));
    }
    Ok(MountPoint::Input(bits))
}

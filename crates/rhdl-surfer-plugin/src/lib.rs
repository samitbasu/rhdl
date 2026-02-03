use color_eyre::{Result, eyre::anyhow};
use std::sync::Mutex;

use camino::Utf8Path;
use extism_pdk::*;
use surfer_translation_types::{
    TranslationPreference, TranslationResult, ValueRepr, VariableInfo, VariableMeta,
    VariableNameInfo, VariableValue, WaveSource, plugin_types::TranslateParams,
};

use crate::{
    context::RHDLContext,
    utils::{color_for_binary_representation, trace_type_to_variable_info},
};

pub mod context;
pub mod trace_string;
pub mod translate;
pub mod utils;

mod host {
    use extism_pdk::host_fn;

    #[host_fn]
    extern "ExtismHost" {
        // TODO - get an incremental read capability here.
        pub fn read_file(filename: String) -> Vec<u8>;
        pub fn file_exists(filename: String) -> bool;
    }
}

fn read_file(filename: &Utf8Path) -> Result<Vec<u8>> {
    unsafe { host::read_file(filename.to_string()) }
        .map_err(|e| anyhow!("Failed to read {filename}. {e}"))
}

fn file_exists(filename: &Utf8Path) -> Result<bool> {
    unsafe { host::file_exists(filename.to_string()) }
        .map_err(|e| anyhow!("Failed to check if {filename} exists {e}"))
}

trait ResultExt<T> {
    fn handle(self) -> FnResult<T>;
}
impl<T> ResultExt<T> for Result<T> {
    fn handle(self) -> FnResult<T> {
        match self {
            Ok(r) => Ok(r),
            Err(e) => Err(WithReturnCode::new(
                extism_pdk::Error::msg(format!("{e:#}")),
                1,
            )),
        }
    }
}

static STATE: Mutex<Option<RHDLContext>> = Mutex::new(None);

#[plugin_fn]
pub fn new() -> FnResult<()> {
    Ok(())
}

#[plugin_fn]
pub fn name() -> FnResult<String> {
    Ok("RHDL (WASM)".to_string())
}

#[plugin_fn]
pub fn set_wave_source(Json(wave_source): Json<Option<WaveSource>>) -> FnResult<()> {
    extism_pdk::info!("Wave source set to: {:?}", wave_source);
    if let Some(WaveSource::File(f)) = wave_source {
        let path = Utf8Path::new(&f);
        let translator = RHDLContext::new(path).handle()?;
        STATE.lock().unwrap().replace(translator);
    } else {
        extism_pdk::info!("Disabling RHDL translator: no wave source file provided");
        drop(STATE.lock().unwrap().take());
        return Ok(());
    }
    Ok(())
}

#[plugin_fn]
pub fn translate(
    TranslateParams { variable, value }: TranslateParams,
) -> FnResult<TranslationResult> {
    let guard = STATE.lock().unwrap();
    let Some(ctx) = guard.as_ref() else {
        return Err(anyhow!(
            "RHDL translator not initialized. Did you set the wave source?"
        ))
        .handle();
    };
    if let Some(ty) = ctx.lookup(&variable.var) {
        return ctx.decode(&value, ty).handle();
    }
    let str_value = match value {
        VariableValue::BigUint(v) => {
            format!(
                "{v:0width$b}",
                width = variable.num_bits.unwrap_or(1) as usize
            )
        }
        VariableValue::String(s) => s.clone(),
    };
    Ok(TranslationResult {
        kind: color_for_binary_representation(&str_value),
        val: ValueRepr::String(str_value),
        subfields: vec![],
    })
}

#[plugin_fn]
pub fn variable_info(variable: VariableMeta<(), ()>) -> FnResult<VariableInfo> {
    let guard = STATE.lock().unwrap();
    let Some(ctx) = guard.as_ref() else {
        return Err(anyhow!(
            "RHDL translator not initialized. Did you set the wave source?"
        ))
        .handle();
    };
    Ok(ctx
        .info(&variable.var)
        .cloned()
        .unwrap_or(VariableInfo::Bits))
}

#[plugin_fn]
pub fn translates(meta: VariableMeta<(), ()>) -> FnResult<TranslationPreference> {
    let mut guard = STATE.lock().unwrap();
    let Some(ctx) = guard.as_mut() else {
        return Err(anyhow!(
            "RHDL translator not initialized. Did you set the wave source?"
        ))
        .handle();
    };
    if ctx.lookup(&meta.var).is_some() {
        return Ok(TranslationPreference::Prefer);
    }
    if ctx.is_skipped(&meta.var) {
        return Ok(TranslationPreference::No);
    }
    let name = meta.var.name.to_string();
    let path = &meta.var.path;
    let full_name = [&path.strs[..], &[name]].concat().join(".");
    if let Some(ty) = ctx.rtt(&full_name) {
        ctx.bind(meta.var, full_name, trace_type_to_variable_info(ty));
        return Ok(TranslationPreference::Prefer);
    }
    ctx.skip_var(meta.var);
    Ok(TranslationPreference::No)
}

#[plugin_fn]
pub fn variable_name_info(
    Json(_variable): Json<VariableMeta<(), ()>>,
) -> FnResult<Option<VariableNameInfo>> {
    Ok(None)
}

#[plugin_fn]
pub fn reload() -> FnResult<()> {
    Ok(())
}

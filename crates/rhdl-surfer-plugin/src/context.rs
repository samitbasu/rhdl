use camino::Utf8Path;
use color_eyre::{
    Result,
    eyre::{Context, bail},
};
use rhdl_trace_type::{RTT, TraceType};
use std::collections::{BTreeMap, HashMap, HashSet};
use surfer_translation_types::{
    TranslationResult, ValueKind, VariableInfo, VariableRef, VariableValue,
};

use crate::{file_exists, read_file, trace_string::TraceString, utils::trace_type_width_in_bits};

#[derive(Default)]
pub struct RHDLContext {
    rtt: BTreeMap<String, rhdl_trace_type::TraceType>,
    symbols: HashMap<VariableRef<(), ()>, String>,
    info: HashMap<VariableRef<(), ()>, VariableInfo>,
    skip_list: HashSet<VariableRef<(), ()>>,
}

impl RHDLContext {
    pub fn new(wave_file: &Utf8Path) -> Result<Self> {
        let rtt_path = wave_file.with_added_extension("rhdl");
        if !file_exists(&rtt_path)? {
            bail!(
                "Disabling RHDL translator: cannot retrieve rhdl info file: {}",
                rtt_path
            );
        }
        let rtt_data =
            read_file(&rtt_path).with_context(|| format!("Reading RHDL RTT file: {}", rtt_path))?;
        let RTT::TraceInfo(rtt) = ron::de::from_bytes::<RTT>(&rtt_data)
            .with_context(|| format!("Parsing RHDL RTT file: {}", rtt_path))?;
        Ok(RHDLContext {
            rtt,
            ..Default::default()
        })
    }
    pub fn decode(&self, value: &VariableValue, ty: &TraceType) -> Result<TranslationResult> {
        let width = trace_type_width_in_bits(ty);
        let val_vcd_raw: TraceString = match value {
            VariableValue::BigUint(v) => (v, width).into(),
            VariableValue::String(s) => (s.as_str(), width).into(),
        };
        crate::translate::translate_raw(&val_vcd_raw.0[..], ty, ValueKind::Normal)
    }
    pub fn lookup<'a>(&'a self, var: &VariableRef<(), ()>) -> Option<&'a TraceType> {
        if let Some(name) = self.symbols.get(var)
            && let Some(ty) = self.rtt.get(name)
        {
            Some(ty)
        } else {
            None
        }
    }
    pub fn info<'a>(&'a self, var: &VariableRef<(), ()>) -> Option<&'a VariableInfo> {
        self.info.get(var)
    }
    pub fn is_skipped(&self, var: &VariableRef<(), ()>) -> bool {
        self.skip_list.contains(var)
    }
    pub fn rtt(&self, name: &str) -> Option<&TraceType> {
        self.rtt.get(name)
    }
    pub fn skip_var(&mut self, var: VariableRef<(), ()>) {
        self.skip_list.insert(var);
    }
    pub fn bind(&mut self, var: VariableRef<(), ()>, name: String, info: VariableInfo) {
        self.symbols.insert(var.clone(), name);
        self.info.insert(var, info);
    }
}

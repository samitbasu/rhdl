use rhdl::{core::RHDLError, prelude::ExportError};
use serde::Serialize;
use tinytemplate::TinyTemplate;

pub fn tt_render<S: Serialize>(template: &'static str, context: S) -> Result<String, RHDLError> {
    let mut tt = TinyTemplate::new();
    tt.add_template("template", template)
        .map_err(|err| RHDLError::ExportError(ExportError::TemplateError(err)))?;
    tt.render("template", &context)
        .map_err(|err| RHDLError::ExportError(ExportError::TemplateError(err)))
}

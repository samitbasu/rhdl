use crate::error::BspError;
use serde::Serialize;
use tinytemplate::TinyTemplate;

pub fn tt_render<S: Serialize>(template: &'static str, context: S) -> Result<String, BspError> {
    let mut tt = TinyTemplate::new();
    tt.add_template("template", template)?;
    tt.render("template", &context).map_err(|err| err.into())
}

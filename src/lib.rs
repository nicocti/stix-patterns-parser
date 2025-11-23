use pyo3::prelude::*;

pub mod ast;
pub mod bindings;
pub mod parser;

#[pymodule(name = "stix_patterns_parser")]
fn pythonapi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    bindings::register(m)?;
    Ok(())
}

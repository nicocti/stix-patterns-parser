//! PyO3 bindings registration for STIX pattern parser.

use pyo3::prelude::*;

use crate::ast::{
    BooleanOp, Comparison, ComparisonOp, CompositeComparison, CompositePattern, ObjectPath,
    ObservationOp, PathComponent, QualifiedPattern, UnaryOp,
};
use crate::parser;

#[pyfunction]
pub fn parse(py: Python<'_>, pattern: &str) -> PyResult<Py<PyAny>> {
    let ast = parser::parse_pattern(pattern)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    ast.to_pyobject(py)
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ComparisonOp>()?;
    m.add_class::<UnaryOp>()?;
    m.add_class::<BooleanOp>()?;
    m.add_class::<ObservationOp>()?;
    m.add_class::<PathComponent>()?;
    m.add_class::<ObjectPath>()?;
    m.add_class::<Comparison>()?;
    m.add_class::<CompositeComparison>()?;
    m.add_class::<CompositePattern>()?;
    m.add_class::<QualifiedPattern>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}

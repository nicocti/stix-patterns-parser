use chrono::{DateTime, Datelike, Timelike, Utc};
use pyo3::prelude::*;
use pyo3::types::PyDateTime;

#[pyclass(frozen, eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOp {
    #[pyo3(name = "EQ")]
    Eq,
    #[pyo3(name = "NEQ")]
    Neq,
    #[pyo3(name = "GT")]
    Gt,
    #[pyo3(name = "LT")]
    Lt,
    #[pyo3(name = "GE")]
    Ge,
    #[pyo3(name = "LE")]
    Le,
    #[pyo3(name = "IN")]
    In,
    #[pyo3(name = "LIKE")]
    Like,
    #[pyo3(name = "MATCHES")]
    Matches,
    #[pyo3(name = "ISSUBSET")]
    IsSubset,
    #[pyo3(name = "ISSUPERSET")]
    IsSuperset,
}

#[pymethods]
impl ComparisonOp {
    fn __repr__(&self) -> &'static str {
        match self {
            Self::Eq => "ComparisonOp.EQ",
            Self::Neq => "ComparisonOp.NEQ",
            Self::Gt => "ComparisonOp.GT",
            Self::Lt => "ComparisonOp.LT",
            Self::Ge => "ComparisonOp.GE",
            Self::Le => "ComparisonOp.LE",
            Self::In => "ComparisonOp.IN",
            Self::Like => "ComparisonOp.LIKE",
            Self::Matches => "ComparisonOp.MATCHES",
            Self::IsSubset => "ComparisonOp.ISSUBSET",
            Self::IsSuperset => "ComparisonOp.ISSUPERSET",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "!=",
            Self::Gt => ">",
            Self::Lt => "<",
            Self::Ge => ">=",
            Self::Le => "<=",
            Self::In => "IN",
            Self::Like => "LIKE",
            Self::Matches => "MATCHES",
            Self::IsSubset => "ISSUBSET",
            Self::IsSuperset => "ISSUPERSET",
        }
    }
}

#[pyclass(frozen, eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    #[pyo3(name = "EXISTS")]
    Exists,
}

#[pymethods]
impl UnaryOp {
    fn __repr__(&self) -> &'static str {
        "UnaryOp.EXISTS"
    }

    #[getter]
    fn value(&self) -> &'static str {
        "EXISTS"
    }
}

#[pyclass(frozen, eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BooleanOp {
    #[default]
    #[pyo3(name = "AND")]
    And,
    #[pyo3(name = "OR")]
    Or,
}

#[pymethods]
impl BooleanOp {
    fn __repr__(&self) -> &'static str {
        match self {
            Self::And => "BooleanOp.AND",
            Self::Or => "BooleanOp.OR",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }
}

#[pyclass(frozen, eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservationOp {
    #[pyo3(name = "AND")]
    And,
    #[pyo3(name = "OR")]
    Or,
    #[pyo3(name = "FOLLOWEDBY")]
    FollowedBy,
}

#[pymethods]
impl ObservationOp {
    fn __repr__(&self) -> &'static str {
        match self {
            Self::And => "ObservationOp.AND",
            Self::Or => "ObservationOp.OR",
            Self::FollowedBy => "ObservationOp.FOLLOWEDBY",
        }
    }

    #[getter]
    fn value(&self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
            Self::FollowedBy => "FOLLOWEDBY",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ListIndex {
    Index(u32),
    Star,
}

#[pyclass(frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathComponent {
    #[pyo3(get)]
    pub property: String,
    index: Option<ListIndex>,
}

#[pymethods]
impl PathComponent {
    #[getter]
    fn index(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.index.as_ref().map(|idx| match idx {
            ListIndex::Index(i) => (*i).into_pyobject(py).unwrap().into_any().unbind(),
            ListIndex::Star => "*".into_pyobject(py).unwrap().into_any().unbind(),
        })
    }

    fn __repr__(&self) -> String {
        match &self.index {
            Some(ListIndex::Index(i)) => {
                format!("PathComponent(property={:?}, index={})", self.property, i)
            }
            Some(ListIndex::Star) => {
                format!("PathComponent(property={:?}, index='*')", self.property)
            }
            None => format!("PathComponent(property={:?}, index=None)", self.property),
        }
    }
}

impl PathComponent {
    #[must_use]
    pub fn new(property: String, index: Option<ListIndex>) -> Self {
        Self { property, index }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPath {
    #[pyo3(get)]
    pub object_type: String,
    pub property_path: Vec<PathComponent>,
}

#[pymethods]
impl ObjectPath {
    #[getter]
    fn property_path(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.property_path.clone().into_pyobject(py)?.unbind())
    }

    fn __repr__(&self) -> String {
        format!("ObjectPath(object_type={:?}, ...)", self.object_type)
    }
}

impl ObjectPath {
    #[must_use]
    pub fn new(object_type: String, property_path: Vec<PathComponent>) -> Self {
        Self {
            object_type,
            property_path,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StixValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Timestamp(DateTime<Utc>),
    Hex(String),
    Binary(String),
}

impl StixValue {
    pub fn to_pyobject(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self {
            Self::String(s) | Self::Hex(s) | Self::Binary(s) => {
                Ok(s.into_pyobject(py)?.into_any().unbind())
            }
            Self::Int(i) => Ok((*i).into_pyobject(py)?.into_any().unbind()),
            Self::Float(f) => Ok((*f).into_pyobject(py)?.into_any().unbind()),
            Self::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any().unbind()),
            Self::Timestamp(dt) => Ok(datetime_to_pyobject(dt, py)?.into_any()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonRhs {
    Value(StixValue),
    List(Vec<StixValue>),
}

impl ComparisonRhs {
    pub fn to_pyobject(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self {
            Self::Value(v) => v.to_pyobject(py),
            Self::List(values) => {
                let py_values: PyResult<Vec<_>> =
                    values.iter().map(|v| v.to_pyobject(py)).collect();
                Ok(py_values?.into_pyobject(py)?.into_any().unbind())
            }
        }
    }
}

impl From<StixValue> for ComparisonRhs {
    fn from(value: StixValue) -> Self {
        Self::Value(value)
    }
}

impl From<Vec<StixValue>> for ComparisonRhs {
    fn from(values: Vec<StixValue>) -> Self {
        Self::List(values)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComparisonOperator {
    Comparison(ComparisonOp),
    Unary(UnaryOp),
}

impl ComparisonOperator {
    fn to_pyobject(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self {
            Self::Comparison(op) => Ok((*op).into_pyobject(py)?.into_any().unbind()),
            Self::Unary(op) => Ok((*op).into_pyobject(py)?.into_any().unbind()),
        }
    }
}

impl From<ComparisonOp> for ComparisonOperator {
    fn from(op: ComparisonOp) -> Self {
        Self::Comparison(op)
    }
}

impl From<UnaryOp> for ComparisonOperator {
    fn from(op: UnaryOp) -> Self {
        Self::Unary(op)
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct Comparison {
    object_path: ObjectPath,
    op: ComparisonOperator,
    constant: Option<ComparisonRhs>,
    #[pyo3(get)]
    pub negated: bool,
}

#[pymethods]
impl Comparison {
    #[getter]
    fn object_path(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self
            .object_path
            .clone()
            .into_pyobject(py)?
            .into_any()
            .unbind())
    }

    #[getter]
    fn op(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.op.to_pyobject(py)
    }

    #[getter]
    fn constant(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.constant
            .as_ref()
            .map(|r| r.to_pyobject(py))
            .transpose()
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        let op_repr = self
            .op(py)
            .and_then(|o| o.bind(py).repr().map(|s| s.to_string()))
            .unwrap_or_else(|_| "?".to_string());
        format!("Comparison(op={}, negated={})", op_repr, self.negated)
    }
}

impl Comparison {
    #[must_use]
    pub fn new(
        lhs: ObjectPath,
        op: impl Into<ComparisonOperator>,
        rhs: Option<ComparisonRhs>,
        negated: bool,
    ) -> Self {
        Self {
            object_path: lhs,
            op: op.into(),
            constant: rhs,
            negated,
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct CompositeComparison {
    left: Box<ComparisonExpr>,
    #[pyo3(get)]
    pub op: BooleanOp,
    right: Box<ComparisonExpr>,
}

#[pymethods]
impl CompositeComparison {
    #[getter]
    fn left(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.left.to_pyobject(py)
    }

    #[getter]
    fn right(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.right.to_pyobject(py)
    }

    fn __repr__(&self) -> String {
        format!("CompositeComparison(op={:?}, ...)", self.op)
    }
}

impl CompositeComparison {
    #[must_use]
    pub fn new(left: ComparisonExpr, op: BooleanOp, right: ComparisonExpr) -> Self {
        Self {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ComparisonExpr {
    Single(Comparison),
    Composite(CompositeComparison),
}

impl ComparisonExpr {
    pub fn to_pyobject(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self {
            Self::Single(c) => Ok(c.clone().into_pyobject(py)?.into_any().unbind()),
            Self::Composite(c) => Ok(c.clone().into_pyobject(py)?.into_any().unbind()),
        }
    }
}

impl From<Comparison> for ComparisonExpr {
    fn from(c: Comparison) -> Self {
        Self::Single(c)
    }
}

impl From<CompositeComparison> for ComparisonExpr {
    fn from(c: CompositeComparison) -> Self {
        Self::Composite(c)
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct CompositePattern {
    left: Box<PatternExpr>,
    #[pyo3(get)]
    pub op: ObservationOp,
    right: Box<PatternExpr>,
}

#[pymethods]
impl CompositePattern {
    #[getter]
    fn left(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.left.to_pyobject(py)
    }

    #[getter]
    fn right(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.right.to_pyobject(py)
    }

    fn __repr__(&self) -> String {
        format!("CompositePattern(op={:?}, ...)", self.op)
    }
}

impl CompositePattern {
    #[must_use]
    pub fn new(left: PatternExpr, op: ObservationOp, right: PatternExpr) -> Self {
        Self {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

#[pyclass(frozen)]
#[derive(Debug, Clone)]
pub struct QualifiedPattern {
    pattern: Box<PatternExpr>,
    #[pyo3(get)]
    pub repeat: Option<u32>,
    #[pyo3(get)]
    pub within: Option<f64>,
    start: Option<DateTime<Utc>>,
    stop: Option<DateTime<Utc>>,
}

#[pymethods]
impl QualifiedPattern {
    #[getter]
    fn pattern(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.pattern.to_pyobject(py)
    }

    #[getter]
    fn start(&self, py: Python<'_>) -> PyResult<Option<Py<PyDateTime>>> {
        self.start
            .as_ref()
            .map(|dt| datetime_to_pyobject(dt, py))
            .transpose()
    }

    #[getter]
    fn stop(&self, py: Python<'_>) -> PyResult<Option<Py<PyDateTime>>> {
        self.stop
            .as_ref()
            .map(|dt| datetime_to_pyobject(dt, py))
            .transpose()
    }

    fn __repr__(&self) -> String {
        format!(
            "QualifiedPattern(repeat={:?}, within={:?}, ...)",
            self.repeat, self.within
        )
    }
}

impl QualifiedPattern {
    #[must_use]
    pub fn new(
        pattern: PatternExpr,
        repeat: Option<u32>,
        within: Option<f64>,
        start: Option<DateTime<Utc>>,
        stop: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            pattern: Box::new(pattern),
            repeat,
            within,
            start,
            stop,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PatternExpr {
    Comparison(ComparisonExpr),
    Composite(CompositePattern),
    Qualified(QualifiedPattern),
}

impl PatternExpr {
    pub fn to_pyobject(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self {
            Self::Comparison(c) => c.to_pyobject(py),
            Self::Composite(c) => Ok(c.clone().into_pyobject(py)?.into_any().unbind()),
            Self::Qualified(q) => Ok(q.clone().into_pyobject(py)?.into_any().unbind()),
        }
    }
}

impl From<ComparisonExpr> for PatternExpr {
    fn from(c: ComparisonExpr) -> Self {
        Self::Comparison(c)
    }
}

impl From<CompositePattern> for PatternExpr {
    fn from(c: CompositePattern) -> Self {
        Self::Composite(c)
    }
}

impl From<QualifiedPattern> for PatternExpr {
    fn from(q: QualifiedPattern) -> Self {
        Self::Qualified(q)
    }
}

fn datetime_to_pyobject(dt: &DateTime<Utc>, py: Python<'_>) -> PyResult<Py<PyDateTime>> {
    let datetime = PyDateTime::new(
        py,
        dt.year(),
        dt.month() as u8,
        dt.day() as u8,
        dt.hour() as u8,
        dt.minute() as u8,
        dt.second() as u8,
        dt.timestamp_subsec_micros(),
        None,
    )?;
    Ok(datetime.unbind())
}

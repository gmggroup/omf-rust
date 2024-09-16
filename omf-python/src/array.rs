use omf::{array_type, Array};
use pyo3::prelude::*;

#[pyclass(name = "ArrayVertex")]
pub struct PyArrayVertex {
    pub inner: Array<array_type::Vertex>,
}

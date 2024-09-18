use omf::{array_type, Array};
use pyo3::prelude::*;

#[pyclass(name = "ArrayVertex")]
pub struct PyArrayVertex {
    pub inner: Array<array_type::Vertex>,
}

#[pyclass(name = "ArrayIndex")]
pub struct PyArrayIndex {
    pub inner: Array<array_type::Index>,
}

#[pyclass(name = "ArrayTriangle")]
pub struct PyArrayTriangle {
    pub inner: Array<array_type::Triangle>,
}

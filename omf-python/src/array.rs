use omf::{array_type, Array};
use pyo3::prelude::*;

#[pyclass(name = "VertexArray")]
pub struct PyVertexArray {
    pub inner: Array<array_type::Vertex>,
}
#[pymethods]
impl PyVertexArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.inner.item_count()
    }
}

#[pyclass(name = "IndexArray")]
pub struct PyIndexArray {
    pub inner: Array<array_type::Index>,
}
#[pymethods]
impl PyIndexArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.inner.item_count()
    }
}

#[pyclass(name = "ArrayTriangle")]
pub struct PyArrayTriangle {
    pub inner: Array<array_type::Triangle>,
}
#[pymethods]
impl PyArrayTriangle {
    #[getter]
    fn item_count(&self) -> u64 {
        self.inner.item_count()
    }
}

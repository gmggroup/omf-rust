use omf::{array_type, Array};
use pyo3::prelude::*;

#[pyclass(name = "VertexArray")]
pub struct PyVertexArray(pub Array<array_type::Vertex>);
#[pymethods]
impl PyVertexArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[pyclass(name = "IndexArray")]
pub struct PyIndexArray(pub Array<array_type::Index>);
#[pymethods]
impl PyIndexArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[pyclass(name = "TriangleArray")]
pub struct PyTriangleArray(pub Array<array_type::Triangle>);
#[pymethods]
impl PyTriangleArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

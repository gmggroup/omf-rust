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

#[pyclass(name = "SegmentArray")]
pub struct PySegmentArray(pub Array<array_type::Segment>);

#[pymethods]
impl PySegmentArray {
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

#[pyclass(name = "ColorArray")]
pub struct PyColorArray(pub Array<array_type::Color>);
#[pymethods]
impl PyColorArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[pyclass(name = "NameArray")]
pub struct PyNameArray(pub Array<array_type::Name>);
#[pymethods]
impl PyNameArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

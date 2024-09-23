use omf::{array_type, Array};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "VertexArray")]
pub struct PyVertexArray(pub Array<array_type::Vertex>);

#[gen_stub_pymethods]
#[pymethods]
impl PyVertexArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "SegmentArray")]
pub struct PySegmentArray(pub Array<array_type::Segment>);

#[gen_stub_pymethods]
#[pymethods]
impl PySegmentArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "IndexArray")]
pub struct PyIndexArray(pub Array<array_type::Index>);

#[gen_stub_pymethods]
#[pymethods]
impl PyIndexArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "TriangleArray")]
pub struct PyTriangleArray(pub Array<array_type::Triangle>);

#[gen_stub_pymethods]
#[pymethods]
impl PyTriangleArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "ColorArray")]
pub struct PyColorArray(pub Array<array_type::Color>);

#[gen_stub_pymethods]
#[pymethods]
impl PyColorArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "NameArray")]
pub struct PyNameArray(pub Array<array_type::Name>);

#[gen_stub_pymethods]
#[pymethods]
impl PyNameArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "GradientArray")]
pub struct PyGradientArray(pub Array<array_type::Gradient>);
#[gen_stub_pymethods]
#[pymethods]
impl PyGradientArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "ImageArray")]
pub struct PyImageArray(pub Array<array_type::Image>);

#[pymethods]
#[gen_stub_pymethods]
impl PyImageArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "TextureCoordinatesArray")]
pub struct PyTextureCoordinatesArray(pub Array<array_type::Texcoord>);

#[gen_stub_pymethods]
#[pymethods]
impl PyTextureCoordinatesArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "NumberArray")]
/// Nullable number values, floating-point or signed integer.
pub struct PyNumberArray(pub Array<array_type::Number>);
#[pymethods]
impl PyNumberArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

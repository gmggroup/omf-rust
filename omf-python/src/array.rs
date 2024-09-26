use omf::{array_type, Array};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "VertexArray")]
/// Vertex locations in 3D.
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
/// Line segments as indices into a vertex array.
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
/// Nullable category index values.
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
/// Triangles as indices into a vertex array.
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
/// Nullable colors.
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
/// Non-nullable category names.
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
/// Non-nullable colormap or category colors.
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
/// An image in PNG or JPEG encoding.
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
/// UV texture coordinates.
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

#[gen_stub_pyclass]
#[pyclass(name = "VectorArray")]
/// Nullable 2D or 3D vectors.
pub struct PyVectorArray(pub Array<array_type::Vector>);
#[pymethods]
impl PyVectorArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "TextArray")]
/// Nullable text.
pub struct PyTextArray(pub Array<array_type::Text>);
#[pymethods]
impl PyTextArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "BooleanArray")]
/// Nullable booleans.
pub struct PyBooleanArray(pub Array<array_type::Boolean>);
#[pymethods]
impl PyBooleanArray {
    #[getter]
    fn item_count(&self) -> u64 {
        self.0.item_count()
    }
}

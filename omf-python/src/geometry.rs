use crate::array::{PySegmentArray, PyTriangleArray, PyVertexArray};
use omf::{LineSet, PointSet, Surface};

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "PointSet")]
/// Point set geometry.
pub struct PyPointSet(pub PointSet);

#[gen_stub_pymethods]
#[pymethods]
impl PyPointSet {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.0.origin
    }

    #[getter]
    fn vertices(&self) -> PyVertexArray {
        PyVertexArray(self.0.vertices.clone())
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "LineSet")]
/// A set of line segments.
pub struct PyLineSet(pub LineSet);

#[gen_stub_pymethods]
#[pymethods]
impl PyLineSet {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.0.origin
    }

    #[getter]
    fn vertices(&self) -> PyVertexArray {
        PyVertexArray(self.0.vertices.clone())
    }

    #[getter]
    fn segments(&self) -> PySegmentArray {
        PySegmentArray(self.0.segments.clone())
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Surface")]
/// A surface made up of triangles.
pub struct PySurface(pub Surface);

#[gen_stub_pymethods]
#[pymethods]
impl PySurface {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.0.origin
    }

    #[getter]
    fn vertices(&self) -> PyVertexArray {
        PyVertexArray(self.0.vertices.clone())
    }

    #[getter]
    fn triangles(&self) -> PyTriangleArray {
        PyTriangleArray(self.0.triangles.clone())
    }
}

//use omf::data::Vertices;

use crate::array::{PySegmentArray, PyTriangleArray, PyVertexArray};
use omf::{Geometry, LineSet, PointSet, Surface};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "Geometry")]
pub struct PyGeometry(pub Geometry);

#[gen_stub_pymethods]
#[pymethods]
impl PyGeometry {
    fn type_name(&self) -> String {
        //self.0.type_name().clone()
        match &self.0 {
            Geometry::PointSet(_) => "PointSet".to_string(),
            Geometry::LineSet(_) => "LineSet".to_string(),
            Geometry::Surface(_) => "Surface".to_string(),
            Geometry::GridSurface(_) => "GridSurface".to_string(),
            Geometry::Composite(_) => "Composite".to_string(),
            Geometry::BlockModel(b) if b.has_subblocks() => "BlockModel(sub-blocked)".to_string(),
            Geometry::BlockModel(_) => "BlockModel".to_string(),
        }
    }

    fn get_object(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.0 {
            Geometry::PointSet(point_set) => Ok(PyPointSet(point_set.clone()).into_py(py)),
            Geometry::LineSet(line_set) => Ok(PyLineSet(line_set.clone()).into_py(py)),
            Geometry::Surface(surface) => Ok(PySurface(surface.clone()).into_py(py)),
            _ => Err(PyValueError::new_err(format!(
                "Geometry {} is not supported",
                self.type_name()
            ))),
        }
    }
}

#[pyclass(name = "PointSet")]
pub struct PyPointSet(PointSet);

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

#[pyclass(name = "LineSet")]
pub struct PyLineSet(LineSet);

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

#[pyclass(name = "Surface")]
pub struct PySurface(Surface);

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

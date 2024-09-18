//use omf::data::Vertices;

use crate::array::{PyTriangleArray, PyVertexArray};
use omf::{Geometry, PointSet, Surface};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(name = "Geometry")]
pub struct PyGeometry(pub Geometry);

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
    fn vertices(&self) -> PyResult<PyVertexArray> {
        Ok(PyVertexArray(self.0.vertices.clone()))
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
    fn vertices(&self) -> PyResult<PyVertexArray> {
        Ok(PyVertexArray(self.0.vertices.clone()))
    }

    #[getter]
    fn triangles(&self) -> PyResult<PyTriangleArray> {
        Ok(PyTriangleArray(self.0.triangles.clone()))
    }
}

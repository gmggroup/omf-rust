//use omf::data::Vertices;

use crate::array::{PyArrayTriangle, PyArrayVertex};
use omf::{Geometry, PointSet, Surface};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(name = "Geometry")]
pub struct PyGeometry {
    pub inner: Geometry,
}

#[pymethods]
impl PyGeometry {
    fn type_name(&self) -> String {
        //self.inner.type_name().clone()
        match &self.inner {
            Geometry::PointSet(_) => "PointSet".to_string(),
            Geometry::LineSet(_) => "LineSet".to_string(),
            Geometry::Surface(_) => "Surface".to_string(),
            Geometry::GridSurface(_) => "GridSurface".to_string(),
            Geometry::Composite(_) => "Composite".to_string(),
            Geometry::BlockModel(b) if b.has_subblocks() => "BlockModel(sub-blocked)".to_string(),
            Geometry::BlockModel(_) => "BlockModel".to_string(),
        }
    }

    fn get_object(&self) -> PyResult<PyPointSet> {
        match &self.inner {
            Geometry::PointSet(point_set) => Ok(PyPointSet {
                inner: point_set.clone(),
            }),
            _ => Err(PyValueError::new_err("Geometry is not a PointSet")),
        }
    }
}

#[pyclass(name = "PointSet")]
pub struct PyPointSet {
    inner: PointSet,
}

#[pymethods]
impl PyPointSet {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.inner.origin
    }

    #[getter]
    fn vertices(&self) -> PyResult<PyArrayVertex> {
        Ok(PyArrayVertex {
            inner: self.inner.vertices.clone(),
        })
    }
}

#[pyclass(name = "Surface")]
pub struct PySurface {
    inner: Surface,
}

#[pymethods]
impl PySurface {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.inner.origin
    }

    #[getter]
    fn vertices(&self) -> PyResult<PyArrayVertex> {
        Ok(PyArrayVertex {
            inner: self.inner.vertices.clone(),
        })
    }

    #[getter]
    fn triangles(&self) -> PyResult<PyArrayTriangle> {
        Ok(PyArrayTriangle {
            inner: self.inner.triangles.clone(),
        })
    }
}

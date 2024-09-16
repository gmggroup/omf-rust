//use omf::data::Vertices;

use crate::file::reader::PyReader;
use omf::{array_type, Geometry, PointSet};
// use omf::{Geometry, PointSet};

use pyo3::exceptions::{PyIOError, PyValueError};
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

    fn get_vertices(&self, reader: &PyReader) -> PyResult<Vec<[f64; 3]>> {
        let vertices = reader
            .inner
            .array_vertices(&self.inner.vertices)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
        vertices
            .map(|result| result.map(|[x, y, z]| [x, y, z]))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    fn vertices_info(&self) -> PyResult<String> {
        let vertices = &self.inner.vertices;
        let vertex_count = vertices.item_count();
        let vertex_type = std::any::type_name::<array_type::Vertex>();

        Ok(format!(
            "Vertices count: {}, Vertex type: {}",
            vertex_count, vertex_type,
        ))
    }
}

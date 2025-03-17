use crate::attribute::PyAttribute;
use crate::block_model::PyBlockModel;
use crate::errors::{OmfJsonException, OmfNotSupportedException};
use crate::geometry::{PyGridSurface, PyLineSet, PyPointSet, PySurface};
use omf::Element;
use omf::Geometry;
use pyo3::{prelude::*, IntoPyObjectExt};
use pyo3_stub_gen::derive::*;
use serde_pyobject::to_pyobject;

#[gen_stub_pyclass]
#[pyclass(name = "Element")]
/// Defines a single “object” or “shape” within the OMF file.
///
/// Each shape has a name plus other optional metadata, a “geometry” that describes a point-set, surface, etc.,
/// and a list of attributes that that exist on that geometry.
pub struct PyElement(pub Element);

#[gen_stub_pymethods]
#[pymethods]
impl PyElement {
    #[getter]
    /// The element name. Names should be non-empty and unique.
    fn name(&self) -> String {
        self.0.name.clone()
    }

    #[getter]
    /// Optional element description.
    fn description(&self) -> String {
        self.0.description.clone()
    }

    #[getter]
    /// Element metadata.
    fn metadata<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        to_pyobject(py, &self.0.metadata).map_err(|e| OmfJsonException::new_err(e.to_string()))
    }

    /// List of attributes, if any.
    fn attributes(&self) -> Vec<PyAttribute> {
        self.0
            .attributes
            .iter()
            .map(|a| PyAttribute(a.clone()))
            .collect()
    }

    /// The geometry of the element.
    fn geometry(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.0.geometry {
            Geometry::PointSet(point_set) => PyPointSet(point_set.clone()).into_py_any(py),
            Geometry::LineSet(line_set) => PyLineSet(line_set.clone()).into_py_any(py),
            Geometry::Surface(surface) => PySurface(surface.clone()).into_py_any(py),
            Geometry::GridSurface(grid_surface) => {
                PyGridSurface(grid_surface.clone()).into_py_any(py)
            }
            Geometry::BlockModel(block_model) => PyBlockModel(block_model.clone()).into_py_any(py),
            unsupported => Err(OmfNotSupportedException::new_err(format!(
                "Geometry type not supported for {unsupported:?}"
            ))),
        }
    }

    #[getter]
    /// Optional solid color.
    fn color(&self) -> Option<(u8, u8, u8, u8)> {
        self.0.color.as_ref().map(|&[r, g, b, a]| (r, g, b, a))
    }
}

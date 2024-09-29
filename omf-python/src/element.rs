use crate::attribute::PyAttribute;
use crate::errors::OmfNotSupportedException;
use crate::geometry::{PyGridSurface, PyLineSet, PyPointSet, PySurface};
use omf::Color;
use omf::Element;
use omf::Geometry;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

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
            Geometry::PointSet(point_set) => Ok(PyPointSet(point_set.clone()).into_py(py)),
            Geometry::LineSet(line_set) => Ok(PyLineSet(line_set.clone()).into_py(py)),
            Geometry::Surface(surface) => Ok(PySurface(surface.clone()).into_py(py)),
            Geometry::GridSurface(grid_surface) => {
                Ok(PyGridSurface(grid_surface.clone()).into_py(py))
            }
            _ => Err(OmfNotSupportedException::new_err(
                "Geometry type not supported",
            )),
        }
    }

    #[getter]
    /// Optional solid color.
    fn color(&self) -> Option<Color> {
        self.0.color
    }
}

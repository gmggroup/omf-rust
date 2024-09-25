use crate::attribute::PyAttribute;
use crate::geometry::PyGeometry;
use omf::Color;
use omf::Element;
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

    #[getter]
    /// The geometry of the element.
    fn geometry(&self) -> PyGeometry {
        PyGeometry(self.0.geometry.clone())
    }

    #[getter]
    /// Optional solid color.
    fn color(&self) -> Option<Color> {
        self.0.color
    }
}

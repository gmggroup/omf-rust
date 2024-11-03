use crate::element::PyElement;
use numpy::PyArray1;
use omf::Project;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "Project")]
/// This is the root element of an OMF file, holding global metadata and a list of Elements that describe the objects or
/// shapes within the file.
pub struct PyProject(pub Project);

#[gen_stub_pymethods]
#[pymethods]
impl PyProject {
    #[getter]
    /// Project name.
    fn name(&self) -> String {
        self.0.name.clone()
    }

    #[getter]
    /// Optional project description.
    fn description(&self) -> String {
        self.0.description.clone()
    }

    #[getter]
    /// Optional EPSG or PROJ local transformation string, default empty.
    ///
    /// Exactly what is supported depends on the application reading the file.
    fn coordinate_reference_system(&self) -> String {
        self.0.coordinate_reference_system.clone()
    }

    #[getter]
    /// Optional unit for distances and locations within the file.
    ///
    /// Typically “meters”, “metres”, “feet”, or empty because the coordinate reference system defines it.
    /// If both are empty then applications may assume meters.
    fn units(&self) -> String {
        self.0.units.clone()
    }

    #[getter]
    /// Optional project origin, default [0, 0, 0].
    ///
    /// Most geometries also have their own origin field. To get the real location add this origin and the geometry origin
    /// to all locations within each element.
    fn origin<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        PyArray1::from_slice_bound(py, &self.0.origin)
    }

    #[getter]
    /// Optional name or email address of the person that created the file, default empty.
    fn author(&self) -> String {
        self.0.author.clone()
    }

    #[getter]
    /// Optional name and version of the application that created the file, default empty.
    fn application(&self) -> String {
        self.0.application.clone()
    }

    /// List of elements.
    fn elements(&self) -> Vec<PyElement> {
        self.0
            .elements
            .iter()
            .map(|e| PyElement(e.clone()))
            .collect()
    }
}

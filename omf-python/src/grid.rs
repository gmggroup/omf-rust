use omf::{Orient2, Vector3};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "Orient2")]
/// Defines the position and orientation of a 2D plane in 3D space.
pub struct PyOrient2(pub Orient2);

#[gen_stub_pymethods]
#[pymethods]
impl PyOrient2 {
    #[getter]
    fn origin(&self) -> Vector3 {
        self.0.origin
    }

    #[getter]
    fn u(&self) -> Vector3 {
        self.0.u
    }

    #[getter]
    fn v(&self) -> Vector3 {
        self.0.v
    }
}

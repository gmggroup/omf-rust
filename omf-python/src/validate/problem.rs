use omf::validate::Problem;
use pyo3::prelude::*;

#[pyclass(name = "Problem")]
pub struct PyProblem(pub Problem);

#[pymethods]
impl PyProblem {
    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

use omf::validate::Problem;
use pyo3::prelude::*;

#[pyclass(name = "Problem")]
pub struct PyProblem {
    pub inner: Problem,
}

#[pymethods]
impl PyProblem {
    fn __str__(&self) -> String {
        self.inner.to_string()
    }
}

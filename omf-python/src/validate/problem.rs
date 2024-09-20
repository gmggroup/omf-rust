use omf::validate::Problem;
use pyo3::prelude::*;

#[pyclass(name = "Problem")]
pub struct PyProblem(pub Problem);

#[pymethods]
impl PyProblem {
    fn __str__(&self) -> String {
        self.0.to_string()
    }

    #[getter]
    fn reason(&self) -> String {
        self.0.reason.to_string()
    }

    #[getter]
    fn type_name(&self) -> String {
        self.0.ty.to_string()
    }

    #[getter]
    fn name(&self) -> Option<String> {
        self.0.name.clone()
    }

    fn is_error(&self) -> bool {
        self.0.is_error()
    }
}

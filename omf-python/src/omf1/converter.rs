use crate::file::reader::PyLimits;
use crate::validate::PyProblem;
use omf::file::{Compression, Limits};
use omf::omf1::detect_open as omf1_detect_open;
use omf::omf1::Converter;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use std::path::Path;

#[pyfunction]
pub fn detect_open(path: String) -> PyResult<bool> {
    let path = Path::new(&path);
    match omf1_detect_open(path) {
        Ok(result) => Ok(result),
        Err(e) => Err(PyErr::new::<PyIOError, _>(e.to_string())),
    }
}

#[pyclass(name = "Converter")]
pub struct PyConverter(pub Converter);

#[pymethods]
impl PyConverter {
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(PyConverter(Converter::new()))
    }

    fn limits(&self) -> PyResult<PyLimits> {
        let limits = self.0.limits();
        Ok(PyLimits {
            json_bytes: limits.json_bytes,
            image_bytes: limits.image_bytes,
            image_dim: limits.image_dim,
            validation: limits.validation,
        })
    }

    fn set_limits(&mut self, limits: &PyLimits) {
        self.0.set_limits(Limits {
            json_bytes: limits.json_bytes,
            image_bytes: limits.image_bytes,
            image_dim: limits.image_dim,
            validation: limits.validation,
        });
    }

    fn compression(&self) -> PyResult<u32> {
        Ok(self.0.compression().level())
    }

    fn set_compression(&mut self, compression: u32) {
        self.0.set_compression(Compression::new(compression));
    }

    fn convert_open(&self, input_path: String, output_path: String) -> PyResult<Vec<PyProblem>> {
        // TODO: handle other errors ?
        let problems = self
            .0
            .convert_open(input_path, output_path)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

        Ok(problems.iter().map(|e| PyProblem(e.clone())).collect())
    }
}

use crate::errors::OmfException;
use crate::file::reader::PyLimits;
use crate::validate::PyProblem;
use omf::file::{Compression, Limits};
use omf::omf1::detect_open as omf1_detect_open;
use omf::omf1::Converter;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

use std::path::Path;

#[gen_stub_pyfunction()]
#[pyfunction]
/// Returns true if the path looks more like OMF1 than OMF2.
pub fn detect_omf1(path: String) -> PyResult<bool> {
    let path = Path::new(&path);
    match omf1_detect_open(path) {
        Ok(result) => Ok(result),
        Err(e) => Err(OmfException::py_err(e)),
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Omf1Converter")]
/// Converts a OMF1 files to OMF2.
///
/// This object allows you to set up the desired parameters then convert one or more files.
pub struct PyOmf1Converter(pub Converter);

#[gen_stub_pymethods]
#[pymethods]
impl PyOmf1Converter {
    #[new]
    pub fn new() -> PyResult<Self> {
        Ok(PyOmf1Converter(Converter::new()))
    }

    /// Returns the current limits.
    fn limits(&self) -> PyResult<PyLimits> {
        let limits = self.0.limits();
        Ok(PyLimits {
            json_bytes: limits.json_bytes,
            image_bytes: limits.image_bytes,
            image_dim: limits.image_dim,
            validation: limits.validation,
        })
    }

    /// Set the limits to use during conversion.
    fn set_limits(&mut self, limits: &PyLimits) {
        self.0.set_limits(Limits {
            json_bytes: limits.json_bytes,
            image_bytes: limits.image_bytes,
            image_dim: limits.image_dim,
            validation: limits.validation,
        });
    }

    /// Returns the current compression level.
    fn compression(&self) -> PyResult<u32> {
        Ok(self.0.compression().level())
    }

    /// Set the compression level to use when writing. Range 0-9.
    fn set_compression(&mut self, compression: u32) {
        self.0.set_compression(Compression::new(compression));
    }

    /// Runs a conversion from one filename to another.
    ///
    /// The output file will be created if it does not exist, and truncated if it does. On success the validation warnings are returned.
    ///
    /// May be called more than once to convert multiple files with the same parameters.
    fn convert(&self, input_path: String, output_path: String) -> PyResult<Vec<PyProblem>> {
        let problems = self
            .0
            .convert_open(input_path, output_path)
            .map_err(OmfException::py_err)?;

        Ok(problems.iter().map(|e| PyProblem(e.clone())).collect())
    }
}

use crate::array::{
    PyColorArray, PyIndexArray, PyNameArray, PySegmentArray, PyTriangleArray, PyVertexArray,
};
use crate::element::PyColor;
use crate::PyProject;
use omf::file::{Limits, Reader};
use std::fs::File;

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

#[pyclass(name = "Limits")]
pub struct PyLimits {
    #[pyo3(get, set)]
    pub json_bytes: Option<u64>,
    #[pyo3(get, set)]
    pub image_bytes: Option<u64>,
    #[pyo3(get, set)]
    pub image_dim: Option<u32>,
    #[pyo3(get, set)]
    pub validation: Option<u32>,
}

#[pymethods]
impl PyLimits {
    #[new]
    pub fn new() -> PyResult<Self> {
        let limits = Limits::default();
        Ok(PyLimits {
            json_bytes: limits.json_bytes,
            image_bytes: limits.image_bytes,
            image_dim: limits.image_dim,
            validation: limits.validation,
        })
    }
}

#[pyclass(name = "Reader")]
pub struct PyReader(Reader);

#[pymethods]
impl PyReader {
    #[new]
    pub fn new(filepath: &str) -> PyResult<Self> {
        let file = File::open(filepath).map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
        let reader = Reader::new(file).map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
        Ok(PyReader(reader))
    }

    #[getter]
    fn project(&self) -> PyResult<PyProject> {
        let (project, problems) = self
            .0
            .project()
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

        if !problems.is_empty() {
            println!("Warnings while reading project: {:?}", problems);
        }

        Ok(PyProject(project))
    }

    pub fn array_vertices(&self, array: &PyVertexArray) -> PyResult<Vec<[f64; 3]>> {
        self.0
            .array_vertices(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    pub fn array_segments(&self, array: &PySegmentArray) -> PyResult<Vec<[u32; 2]>> {
        self.0
            .array_segments(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    pub fn array_indices(&self, array: &PyIndexArray) -> PyResult<Vec<Option<u32>>> {
        self.0
            .array_indices(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<Option<u32>>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    pub fn array_triangles(&self, array: &PyTriangleArray) -> PyResult<Vec<[u32; 3]>> {
        self.0
            .array_triangles(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    pub fn array_color(&self, array: &PyColorArray) -> PyResult<Vec<Option<PyColor>>> {
        self.0
            .array_colors(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .map(|r| r.map(|c| c.map(PyColor)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    pub fn array_names(&self, array: &PyNameArray) -> PyResult<Vec<String>> {
        self.0
            .array_names(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }
}

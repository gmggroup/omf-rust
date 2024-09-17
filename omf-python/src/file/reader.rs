use crate::array::{PyIndexArray, PySegmentArray, PyTriangleArray, PyVertexArray};
use crate::PyProject;
use omf::file::Reader;
use std::fs::File;

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

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
}

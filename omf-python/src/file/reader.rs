use crate::array::{
    PyBooleanArray, PyColorArray, PyGradientArray, PyImageArray, PyIndexArray, PyNameArray,
    PyNumberArray, PySegmentArray, PyTextArray, PyTextureCoordinatesArray, PyTriangleArray,
    PyVectorArray, PyVertexArray,
};
use crate::validate::PyProblem;
use crate::PyProject;
use omf::file::{Limits, Reader};
use omf::Color;
use pyo3::types::PyBytes;
use std::fs::File;

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "Limits")]
/// Memory limits for reading OMF files.
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

#[gen_stub_pymethods]
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

#[gen_stub_pyclass]
#[pyclass(name = "Reader")]
pub struct PyReader(Reader);

#[gen_stub_pymethods]
#[pymethods]
impl PyReader {
    #[new]
    pub fn new(filepath: &str) -> PyResult<Self> {
        let file = File::open(filepath).map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
        let reader = Reader::new(file).map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
        Ok(PyReader(reader))
    }

    /// Reads, validates, and returns the root `Project` object from the file.
    ///
    /// Fails with an error if an IO error occurs, the `json_bytes` limit is exceeded, or validation
    /// fails. Validation warnings are returned alongside the project if successful or included
    /// with the errors if not.
    fn project(&self) -> PyResult<(PyProject, Vec<PyProblem>)> {
        let (project, problems) = self
            .0
            .project()
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

        let problems_array: Vec<PyProblem> =
            problems.iter().map(|e| PyProblem(e.clone())).collect();
        Ok((PyProject(project), problems_array))
    }

    /// Read a Vertex array.
    pub fn array_vertices(&self, array: &PyVertexArray) -> PyResult<Vec<[f64; 3]>> {
        self.0
            .array_vertices(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Segment array.
    pub fn array_segments(&self, array: &PySegmentArray) -> PyResult<Vec<[u32; 2]>> {
        self.0
            .array_segments(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read an Index array.
    pub fn array_indices(&self, array: &PyIndexArray) -> PyResult<Vec<Option<u32>>> {
        self.0
            .array_indices(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<Option<u32>>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Triangle array.
    pub fn array_triangles(&self, array: &PyTriangleArray) -> PyResult<Vec<[u32; 3]>> {
        self.0
            .array_triangles(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Color array.
    pub fn array_color(&self, array: &PyColorArray) -> PyResult<Vec<Option<Color>>> {
        self.0
            .array_colors(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Gradient array.
    pub fn array_gradient(&self, array: &PyGradientArray) -> PyResult<Vec<[u8; 4]>> {
        self.0
            .array_gradient(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Name array.
    pub fn array_names(&self, array: &PyNameArray) -> PyResult<Vec<String>> {
        self.0
            .array_names(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Texcoord array.
    pub fn array_texcoords(&self, array: &PyTextureCoordinatesArray) -> PyResult<Vec<[f64; 2]>> {
        self.0
            .array_texcoords(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read bytes of an Image.
    pub fn image_bytes<'p>(
        &self,
        py: Python<'p>,
        array: &PyImageArray,
    ) -> PyResult<Bound<'p, PyBytes>> {
        self.0
            .array_bytes(&array.0)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
            .map(|b| PyBytes::new_bound(py, &b))
    }

    /// Read a Number array.
    pub fn array_numbers(&self, array: &PyNumberArray) -> PyResult<Vec<f64>> {
        let numbers_f64 = self
            .0
            .array_numbers(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .try_into_f64()
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

        Ok(numbers_f64
            .into_iter()
            .filter_map(|item| match item {
                Ok(Some(value)) => Some(value),
                _ => None,
            })
            .collect())
    }

    /// Read a Vector array.
    pub fn array_vectors(&self, array: &PyVectorArray) -> PyResult<Vec<Option<[f64; 3]>>> {
        self.0
            .array_vectors(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Text array.
    pub fn array_text(&self, array: &PyTextArray) -> PyResult<Vec<Option<String>>> {
        self.0
            .array_text(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }

    /// Read a Boolean array.
    pub fn array_booleans(&self, array: &PyBooleanArray) -> PyResult<Vec<Option<bool>>> {
        self.0
            .array_booleans(&array.0)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))
    }
}

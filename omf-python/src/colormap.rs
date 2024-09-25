use crate::array::{PyBoundaryArray, PyGradientArray};
use omf::NumberColormap;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(name = "NumberColormapContinuous")]
pub struct PyNumberColormapContinuous(pub NumberColormap);

#[pyclass(name = "NumberColormapDiscrete")]
pub struct PyNumberColormapDiscrete(pub NumberColormap);

#[pymethods]
impl PyNumberColormapContinuous {
    #[getter]
    fn gradient(&self) -> PyResult<PyGradientArray> {
        match &self.0 {
            NumberColormap::Continuous { gradient, .. } => Ok(PyGradientArray(gradient.clone())),
            _ => Err(PyValueError::new_err(
                "NumberColormap variant is not supported",
            )),
        }
    }
}

#[pymethods]
impl PyNumberColormapDiscrete {
    #[getter]
    fn boundaries(&self) -> PyResult<Option<PyBoundaryArray>> {
        match &self.0 {
            NumberColormap::Discrete { boundaries, .. } => {
                Ok(Some(PyBoundaryArray(boundaries.clone())))
            }
            _ => Err(PyValueError::new_err(
                "NumberColormap variant is not supported",
            )),
        }
    }

    #[getter]
    fn gradient(&self) -> PyResult<PyGradientArray> {
        match &self.0 {
            NumberColormap::Discrete { gradient, .. } => Ok(PyGradientArray(gradient.clone())),
            _ => Err(PyValueError::new_err(
                "NumberColormap variant is not supported",
            )),
        }
    }
}

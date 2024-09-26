use crate::array::{PyBoundaryArray, PyGradientArray};
use omf::{NumberColormap, NumberRange};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "NumberRangeFloat")]
/// Specifies the minimum and maximum values of a number colormap.
///
/// Values outside this range will use the color at the ends of the gradient.
/// The variant used should match the type of the number array.
pub struct PyNumberRangeFloat(pub NumberRange);

#[gen_stub_pymethods]
#[pymethods]
impl PyNumberRangeFloat {
    #[getter]
    fn min(&self) -> PyResult<f64> {
        match self.0 {
            NumberRange::Float { min, .. } => Ok(min),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
    #[getter]
    fn max(&self) -> PyResult<f64> {
        match self.0 {
            NumberRange::Float { max, .. } => Ok(max),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "NumberRangeInteger")]
/// Specifies the minimum and maximum values of a number colormap.
///
/// Values outside this range will use the color at the ends of the gradient.
/// The variant used should match the type of the number array.
pub struct PyNumberRangeInteger(pub NumberRange);

#[gen_stub_pymethods]
#[pymethods]
impl PyNumberRangeInteger {
    #[getter]
    fn min(&self) -> PyResult<i64> {
        match self.0 {
            NumberRange::Integer { min, .. } => Ok(min),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
    #[getter]
    fn max(&self) -> PyResult<i64> {
        match self.0 {
            NumberRange::Integer { max, .. } => Ok(max),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "NumberRangeDate")]
/// Specifies the minimum and maximum values of a number colormap.
///
/// Values outside this range will use the color at the ends of the gradient.
/// The variant used should match the type of the number array.
pub struct PyNumberRangeDate(pub NumberRange);

#[gen_stub_pymethods]
#[pymethods]
impl PyNumberRangeDate {
    #[getter]
    fn min(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self.0 {
            NumberRange::Date { min, .. } => Ok(min.into_py(py)),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
    #[getter]
    fn max(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self.0 {
            NumberRange::Date { max, .. } => Ok(max.into_py(py)),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "NumberRangeDateTime")]
/// Specifies the minimum and maximum values of a number colormap.
///
/// Values outside this range will use the color at the ends of the gradient.
/// The variant used should match the type of the number array.
pub struct PyNumberRangeDateTime(pub NumberRange);

#[gen_stub_pymethods]
#[pymethods]
impl PyNumberRangeDateTime {
    #[getter]
    fn min(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self.0 {
            NumberRange::DateTime { min, .. } => Ok(min.into_py(py)),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
    #[getter]
    fn max(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self.0 {
            NumberRange::DateTime { max, .. } => Ok(max.into_py(py)),
            _ => Err(PyValueError::new_err(
                "NumberRange variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "NumberColormapContinuous")]
/// A continuous colormap linearly samples a color gradient within a defined range.
///
/// A value X% of way between `min` and `max` should use the color from X% way down
/// gradient. When that X doesn't land directly on a color use the average of
/// the colors on either side, inverse-weighted by the distance to each.
///
/// Values below the minimum use the first color in the gradient array. Values above
/// the maximum use the last.
pub struct PyNumberColormapContinuous(pub NumberColormap);

#[gen_stub_pymethods]
#[pymethods]
impl PyNumberColormapContinuous {
    #[getter]
    /// Value range.
    fn range(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.0 {
            NumberColormap::Continuous { range, .. } => match &range {
                NumberRange::Float { .. } => Ok(PyNumberRangeFloat(*range).into_py(py)),
                NumberRange::Integer { .. } => Ok(PyNumberRangeInteger(*range).into_py(py)),
                NumberRange::Date { .. } => Ok(PyNumberRangeDate(*range).into_py(py)),
                NumberRange::DateTime { .. } => Ok(PyNumberRangeDateTime(*range).into_py(py)),
            },
            _ => Err(PyValueError::new_err(
                "NumberColormap variant is not supported",
            )),
        }
    }

    #[getter]
    /// Array with `Gradient` type storing the smooth color gradient.
    fn gradient(&self) -> PyResult<PyGradientArray> {
        match &self.0 {
            NumberColormap::Continuous { gradient, .. } => Ok(PyGradientArray(gradient.clone())),
            _ => Err(PyValueError::new_err(
                "NumberColormap variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "NumberColormapDiscrete")]
/// A discrete colormap divides the number line into adjacent but non-overlapping
/// ranges and gives a flat color to each range.
///
/// Values above the last boundary use `end_color`.
pub struct PyNumberColormapDiscrete(pub NumberColormap);

#[gen_stub_pymethods]
#[pymethods]
impl PyNumberColormapDiscrete {
    #[getter]
    /// Array with `Boundary` type storing the smooth color gradient, containing the value
    /// and inclusiveness of each boundary. Values must increase along the array.
    /// Boundary values type should match the type of the number array.
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
    /// Array with `Gradient` type storing the colors of the discrete ranges.
    /// Length must be one more than `boundaries`, with the extra color used for values above
    /// the last boundary.
    fn gradient(&self) -> PyResult<PyGradientArray> {
        match &self.0 {
            NumberColormap::Discrete { gradient, .. } => Ok(PyGradientArray(gradient.clone())),
            _ => Err(PyValueError::new_err(
                "NumberColormap variant is not supported",
            )),
        }
    }
}

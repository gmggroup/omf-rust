use omf::{NumberColormap, NumberRange};
use pyo3::{prelude::*, IntoPyObjectExt};
use pyo3_stub_gen::derive::*;

use crate::array::{PyBoundaryArray, PyGradientArray};

macro_rules! number_colormap_field {
    ($self:ident, $variant:ident :: $field:ident) => {
        match &$self.0 {
            NumberColormap::$variant { $field, .. } => $field,
            _ => unreachable!("NumberColormap variant is not supported"),
        }
    };
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
    /// Value range.
    fn range(&self, py: Python<'_>) -> PyResult<PyObject> {
        match *number_colormap_field!(self, Continuous::range) {
            NumberRange::Float { min, max, .. } => (min, max).into_py_any(py),
            NumberRange::Integer { min, max, .. } => (min, max).into_py_any(py),
            NumberRange::Date { min, max, .. } => (min, max).into_py_any(py),
            NumberRange::DateTime { min, max, .. } => (min, max).into_py_any(py),
        }
    }

    #[getter]
    /// Array with `Gradient` type storing the smooth color gradient.
    fn gradient(&self) -> PyGradientArray {
        PyGradientArray(number_colormap_field!(self, Continuous::gradient).clone())
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
    fn boundaries(&self) -> PyBoundaryArray {
        PyBoundaryArray(number_colormap_field!(self, Discrete::boundaries).clone())
    }

    #[getter]
    /// Array with `Gradient` type storing the colors of the discrete ranges.
    /// Length must be one more than `boundaries`, with the extra color used for values above
    /// the last boundary.
    fn gradient(&self) -> PyGradientArray {
        PyGradientArray(number_colormap_field!(self, Discrete::gradient).clone())
    }
}

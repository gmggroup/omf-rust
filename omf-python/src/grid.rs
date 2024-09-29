use omf::{Grid2, Orient2, Vector3};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

use crate::array::PyScalarArray;

#[gen_stub_pyclass]
#[pyclass(name = "Regular")]
#[derive(Clone)]
/// Regularly spaced cells.
pub struct PyGrid2Regular(Grid2);

impl TryFrom<Grid2> for PyGrid2Regular {
    type Error = ();

    fn try_from(value: Grid2) -> Result<Self, Self::Error> {
        match value {
            // A Regular grid can be converted to PyGrid2Regular.
            Grid2::Regular { .. } => Ok(Self(value)),
            _ => Err(()),
        }
    }
}

impl From<PyGrid2Regular> for Grid2 {
    fn from(value: PyGrid2Regular) -> Self {
        value.0
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyGrid2Regular {
    #[getter]
    /// The cell size in the U and V axes. Both must be greater than zero.
    fn size(&self) -> [f64; 2] {
        match self.0 {
            Grid2::Regular { size, .. } => size,
            Grid2::Tensor { .. } => unreachable!(),
        }
    }

    /// Returns the number of cells in each axis.
    fn count(&self) -> [u32; 2] {
        self.0.count()
    }

    /// Returns the total number of cells.
    fn flat_count(&self) -> u64 {
        self.0.flat_count()
    }

    /// Returns the total number of cell corners.
    fn flat_corner_count(&self) -> u64 {
        self.0.flat_corner_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Tensor")]
#[derive(Clone)]
/// Tensor cells, where each row and column can have a different size.
pub struct PyGrid2Tensor(Grid2);

impl TryFrom<Grid2> for PyGrid2Tensor {
    type Error = ();

    fn try_from(value: Grid2) -> Result<Self, Self::Error> {
        match value {
            // A Tensor grid can be converted to PyGrid2Tensor.
            Grid2::Tensor { .. } => Ok(Self(value)),
            _ => Err(()),
        }
    }
}

impl From<PyGrid2Tensor> for Grid2 {
    fn from(value: PyGrid2Tensor) -> Self {
        value.0
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyGrid2Tensor {
    #[getter]
    fn u(&self) -> PyScalarArray {
        match &self.0 {
            Grid2::Regular { .. } => unreachable!(),
            Grid2::Tensor { u, .. } => PyScalarArray(u.clone()),
        }
    }

    #[getter]
    fn v(&self) -> PyScalarArray {
        match &self.0 {
            Grid2::Regular { .. } => unreachable!(),
            Grid2::Tensor { v, .. } => PyScalarArray(v.clone()),
        }
    }

    /// Returns the number of cells in each axis.
    fn count(&self) -> [u32; 2] {
        self.0.count()
    }

    /// Returns the total number of cells.
    fn flat_count(&self) -> u64 {
        self.0.flat_count()
    }

    /// Returns the total number of cell corners.
    fn flat_corner_count(&self) -> u64 {
        self.0.flat_corner_count()
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Orient2")]
/// Defines the position and orientation of a 2D plane in 3D space.
pub struct PyOrient2(pub Orient2);

#[gen_stub_pymethods]
#[pymethods]
impl PyOrient2 {
    #[getter]
    fn origin(&self) -> Vector3 {
        self.0.origin
    }

    #[getter]
    fn u(&self) -> Vector3 {
        self.0.u
    }

    #[getter]
    fn v(&self) -> Vector3 {
        self.0.v
    }
}

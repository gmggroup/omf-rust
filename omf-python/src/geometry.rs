use crate::{
    array::{PyScalarArray, PySegmentArray, PyTriangleArray, PyVertexArray},
    grid::{PyGrid2Regular, PyGrid2Tensor, PyOrient2},
};
use omf::{GridSurface, LineSet, PointSet, Surface};

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "PointSet")]
/// Point set geometry.
pub struct PyPointSet(pub PointSet);

#[gen_stub_pymethods]
#[pymethods]
impl PyPointSet {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.0.origin
    }

    #[getter]
    fn vertices(&self) -> PyVertexArray {
        PyVertexArray(self.0.vertices.clone())
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "LineSet")]
/// A set of line segments.
pub struct PyLineSet(pub LineSet);

#[gen_stub_pymethods]
#[pymethods]
impl PyLineSet {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.0.origin
    }

    #[getter]
    fn vertices(&self) -> PyVertexArray {
        PyVertexArray(self.0.vertices.clone())
    }

    #[getter]
    fn segments(&self) -> PySegmentArray {
        PySegmentArray(self.0.segments.clone())
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Surface")]
/// A surface made up of triangles.
pub struct PySurface(pub Surface);

#[gen_stub_pymethods]
#[pymethods]
impl PySurface {
    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.0.origin
    }

    #[getter]
    fn vertices(&self) -> PyVertexArray {
        PyVertexArray(self.0.vertices.clone())
    }

    #[getter]
    fn triangles(&self) -> PyTriangleArray {
        PyTriangleArray(self.0.triangles.clone())
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "GridSurface")]
/// A surface defined by a 2D grid a height on each grid vertex.
pub struct PyGridSurface(pub GridSurface);

#[gen_stub_pymethods]
#[pymethods]
impl PyGridSurface {
    #[getter]
    /// Position and orientation of the surface.
    fn orient(&self) -> PyOrient2 {
        PyOrient2(self.0.orient)
    }

    #[getter]
    /// 2D grid definition, which can be regular or tensor.
    fn grid(&self, py: Python<'_>) -> PyObject {
        match self.0.grid {
            omf::Grid2::Regular { .. } => PyGrid2Regular::try_from(self.0.grid.clone())
                .expect("conversion from Regular should succeed")
                .into_py(py),
            omf::Grid2::Tensor { .. } => PyGrid2Tensor::try_from(self.0.grid.clone())
                .expect("conversion from Tensor should succeed")
                .into_py(py),
        }
    }

    /// Array with `Scalar` type storing the offset of each grid vertex from the place.
    /// Heights may be positive or negative. Will be absent from flat 2D grids.
    fn heights(&self) -> Option<PyScalarArray> {
        self.0.heights.as_ref().map(|h| PyScalarArray(h.clone()))
    }
}

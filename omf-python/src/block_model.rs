use omf::BlockModel;

use crate::{
    array::{PyFreeformSubblockArray, PyRegularSubblockArray},
    grid::{PyGrid3Regular, PyGrid3Tensor, PyOrient3},
};

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "BlockModel")]
/// Block model geometry with optional sub-blocks.
pub struct PyBlockModel(pub BlockModel);

#[gen_stub_pymethods]
#[pymethods]
impl PyBlockModel {
    #[getter]
    /// Orientation of the block model.
    fn orient(&self) -> PyOrient3 {
        PyOrient3(self.0.orient)
    }

    #[getter]
    /// Block sizes.
    fn grid(&self, py: Python<'_>) -> PyObject {
        match self.0.grid {
            omf::Grid3::Regular { .. } => PyGrid3Regular::try_from(self.0.grid.clone())
                .expect("conversion from Grid3::Regular should succeed")
                .into_py(py),
            omf::Grid3::Tensor { .. } => PyGrid3Tensor::try_from(self.0.grid.clone())
                .expect("conversion from Grid3::Tensor should succeed")
                .into_py(py),
        }
    }

    #[getter]
    /// Optional sub-blocks, which can be regular or free-form divisions of the parent blocks.
    fn subblocks(&self, py: Python<'_>) -> Option<PyObject> {
        match &self.0.subblocks {
            Some(_subblocks) => match _subblocks {
                omf::Subblocks::Regular { subblocks, .. } => {
                    Some(PyRegularSubblockArray(subblocks.clone()).into_py(py))
                }
                omf::Subblocks::Freeform { subblocks, .. } => {
                    Some(PyFreeformSubblockArray(subblocks.clone()).into_py(py))
                }
            },
            None => None,
        }
    }
}

use omf::{BlockModel, SubblockMode, Subblocks};

use crate::{
    array::{PyFreeformSubblockArray, PyRegularSubblockArray},
    grid::{PyGrid3Regular, PyGrid3Tensor, PyOrient3},
};

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass_enum]
#[pyclass(eq, eq_int, name = "SubblockMode")]
#[derive(PartialEq)]
/// An optional mode for regular sub-blocks.
pub enum PySubblockMode {
    Octree,
    Full,
}

#[gen_stub_pyclass]
#[pyclass(name = "RegularSubblocks")]
/// Block model geometry with optional sub-blocks.
pub struct PyRegularSubblocks(pub Subblocks);

#[gen_stub_pymethods]
#[pymethods]
impl PyRegularSubblocks {
    #[getter]
    /// The sub-block grid size.
    ///
    /// Must be greater than zero in all directions. If `mode` is octree then these must also
    /// be powers of two but they don't have to be equal.
    fn count(&self) -> [u32; 3] {
        match self.0 {
            omf::Subblocks::Regular { count, .. } => count,
            _ => unreachable!(),
        }
    }

    #[getter]
    /// If present this further restricts the sub-block layout.
    fn mode(&self) -> Option<PySubblockMode> {
        match &self.0 {
            omf::Subblocks::Regular { mode, .. } => match mode {
                Some(mode) => match mode {
                    SubblockMode::Octree { .. } => Some(PySubblockMode::Octree),
                    SubblockMode::Full { .. } => Some(PySubblockMode::Full),
                },
                None => None,
            },
            _ => unreachable!(),
        }
    }

    #[getter]
    /// Array storing the sub-block parent indices and corners
    /// relative to the sub-block grid within the parent.
    fn subblocks(&self) -> PyRegularSubblockArray {
        match &self.0 {
            omf::Subblocks::Regular { subblocks, .. } => PyRegularSubblockArray(subblocks.clone()),
            _ => unreachable!(),
        }
    }
}

impl TryFrom<Subblocks> for PyRegularSubblocks {
    type Error = ();

    fn try_from(value: Subblocks) -> Result<Self, Self::Error> {
        match value {
            Subblocks::Regular { .. } => Ok(Self(value)),
            _ => Err(()),
        }
    }
}

impl From<PyRegularSubblocks> for Subblocks {
    fn from(value: PyRegularSubblocks) -> Self {
        value.0
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "FreeformSubblocks")]
/// Block model geometry with optional sub-blocks.
pub struct PyFreeformSubblocks(pub Subblocks);

#[gen_stub_pymethods]
#[pymethods]
impl PyFreeformSubblocks {
    #[getter]
    /// Array storing the sub-block parent indices and corners
    /// relative to the parent.
    fn subblocks(&self) -> PyFreeformSubblockArray {
        match &self.0 {
            omf::Subblocks::Freeform { subblocks, .. } => {
                PyFreeformSubblockArray(subblocks.clone())
            }
            _ => unreachable!(),
        }
    }
}

impl TryFrom<Subblocks> for PyFreeformSubblocks {
    type Error = ();

    fn try_from(value: Subblocks) -> Result<Self, Self::Error> {
        match value {
            Subblocks::Freeform { .. } => Ok(Self(value)),
            _ => Err(()),
        }
    }
}

impl From<PyFreeformSubblocks> for Subblocks {
    fn from(value: PyFreeformSubblocks) -> Self {
        value.0
    }
}

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
            Some(subblocks) => match subblocks {
                Subblocks::Regular { .. } => Some(
                    PyRegularSubblocks::try_from(subblocks.clone())
                        .expect("conversion from Subblocks::Regular should succeed")
                        .into_py(py),
                ),
                Subblocks::Freeform { .. } => Some(
                    PyFreeformSubblocks::try_from(subblocks.clone())
                        .expect("conversion from Subblocks::Freeform should succeed")
                        .into_py(py),
                ),
            },
            None => None,
        }
    }
}

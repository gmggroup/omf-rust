use crate::geometry::PyGeometry;
use omf::Element;
use pyo3::prelude::*;

#[pyclass(name = "Element")]
pub struct PyElement {
    pub inner: Element,
}

#[pymethods]
impl PyElement {
    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.inner.name.clone())
    }
    #[getter]
    fn description(&self) -> PyResult<String> {
        Ok(self.inner.description.clone())
    }
    #[getter]
    fn geometry(&self) -> PyResult<PyGeometry> {
        Ok(PyGeometry {
            inner: self.inner.geometry.clone(),
        })
    }
}

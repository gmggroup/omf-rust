use omf::error::Error;
use pyo3::exceptions::PyException;
use pyo3::{create_exception, PyErr};

create_exception!(
    omf_python,
    OmfException,
    PyException,
    "Base class for all OMF exceptions."
);

impl OmfException {
    pub(crate) fn py_err(e: Error) -> PyErr {
        let s = e.to_string();
        OmfException::new_err(s)
    }
}

use omf::error::Error;
use pyo3::exceptions::PyException;
use pyo3::{create_exception, PyErr};

create_exception!(
    omf_python,
    OmfException,
    PyException,
    "Base class for all OMF exceptions."
);

create_exception!(omf_python, OmfIoError, OmfException, "An IO error.");
create_exception!(
    omf_python,
    OmfValidationFailedError,
    OmfException,
    "An error indicating that OMF validation failed."
);

impl OmfException {
    pub(crate) fn py_err(e: Error) -> PyErr {
        let s = e.to_string();
        match e {
            Error::IoError(_) => OmfIoError::new_err(s),
            Error::ValidationFailed(_) => OmfValidationFailedError::new_err(s),
            // Remaining errors are converted to generic OMF exceptions.
            _ => OmfException::new_err(s),
        }
    }
}

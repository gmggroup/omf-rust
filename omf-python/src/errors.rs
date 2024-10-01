use omf::error::Error;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::PyErr;
use pyo3_stub_gen::inventory;
use pyo3_stub_gen::type_info::PyClassInfo;

// Because pyo3-stub-gen doesn't generate doc strings for exceptions
// we manually create PyClassInfo entries with the doc strings.

macro_rules! create_exception_impl {
    ($module: expr, $name: ident, $base: ty, $base_class: expr, $doc: expr) => {
        create_exception!($module, $name, $base);

        inventory::submit! {
            PyClassInfo {
                struct_id: std::any::TypeId::of::<$name>,
                pyclass_name: concat!(stringify!($name), "(", $base_class, ")"),
                module: Some(stringify!($module)),
                members: &[],
                doc: $doc,
            }
        }
    };
}

// NOTE: OmfException needs to be first alphabetically for the exceptions show correctly
// in the Sphinx documentation as that is the order they get added into omf_python.pyi.

create_exception_impl!(
    omf_python,
    OmfException,
    PyException,
    "Exception",
    "Base class for all OMF exceptions."
);

create_exception_impl!(
    omf_python,
    OmfFileIoException,
    OmfException,
    "OmfException",
    "Exception raised when a file IO error occurs."
);
create_exception_impl!(
    omf_python,
    OmfJsonException,
    OmfException,
    "OmfException",
    "Exception raised when a JSON error occurs. Can also be triggered by exceeding the `json_bytes` safety limit."
);
create_exception_impl!(
    omf_python,
    OmfLimitExceededException,
    OmfException,
    "OmfException",
    "Exception raised when a safety limit was exceeded."
);
create_exception_impl!(
    omf_python,
    OmfInvalidDataException,
    OmfException,
    "OmfException",
    "Exception raised when an OMF file contains invalid data."
);

create_exception_impl!(
    omf_python,
    OmfValidationFailedException,
    OmfException,
    "OmfException",
    "Exception raised when an OMF validation error occurs."
);
create_exception_impl!(
    omf_python,
    OmfNotSupportedException,
    OmfException,
    "OmfException",
    "Exception raised when some action is not supported."
);

impl OmfException {
    /// Convert OMF errors to OmfException derived python exceptions
    pub(crate) fn py_err(e: Error) -> PyErr {
        let s = e.to_string();
        match e {
            Error::IoError(_) => OmfFileIoException::new_err(s),
            Error::LimitExceeded(_) => OmfLimitExceededException::new_err(s),
            Error::InvalidData(_) => OmfInvalidDataException::new_err(s),
            Error::ValidationFailed(problems) => {
                OmfValidationFailedException::new_err(problems.to_string())
            }
            Error::DeserializationFailed(_) => OmfJsonException::new_err(s),
            // Remaining errors are converted to generic OMF exceptions.
            _ => OmfException::new_err(s),
        }
    }
}

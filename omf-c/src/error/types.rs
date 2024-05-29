/// Describes how an argument can be invalid.
#[derive(Debug, thiserror::Error)]
pub enum InvalidArg {
    #[error("{0} must not be null")]
    Null(&'static str),
    #[error("{0} must not be null unless {1} is zero")]
    NullArray(&'static str, &'static str),
    #[error("{0} must be UTF-8 encoded")]
    NotUtf8(&'static str),
    #[error("one of the {0} options must be non-null")]
    NoOptionSet(&'static str),
    #[error("invalid handle")]
    Handle,
    #[error("{0}")]
    HandleType(&'static str),
    #[error("invalid enum value")]
    Enum,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Panic: {0}")]
    Panic(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(#[from] InvalidArg),
    #[error("Invalid call: {0}")]
    InvalidCall(String),
    #[error("Error: buffer length {found} does not match expected length {expected}")]
    BufferLengthWrong { found: u64, expected: u64 },
    #[error("Error: '{src}' data type {found:?} does not match expected type {expected:?}")]
    ArrayTypeWrong {
        src: &'static str,
        found: omf::DataType,
        expected: omf::DataType,
    },
    #[error("{message}")]
    External {
        code: i32,
        detail: i32,
        message: String,
    },
}

impl Error {
    pub fn code(&self) -> i32 {
        match self {
            Error::Panic(_) => Status::Panic as i32,
            Error::InvalidArgument(_) => Status::InvalidArgument as i32,
            Error::InvalidCall(_) => Status::InvalidCall as i32,
            Error::BufferLengthWrong { .. } => Status::BufferLengthWrong as i32,
            Error::ArrayTypeWrong { .. } => Status::ArrayTypeWrong as i32,
            Error::External { code, .. } => *code,
        }
    }

    pub fn detail(&self) -> i32 {
        match self {
            Error::External { detail, .. } => *detail,
            _ => 0,
        }
    }
}

impl From<omf::error::Error> for Error {
    fn from(error: omf::error::Error) -> Self {
        use omf::error::Error::*;
        let message = error.to_string();
        let (status, detail) = match error {
            OutOfMemory => (Status::OutOfMemory, 0),
            IoError(e) => (Status::IoError, e.raw_os_error().unwrap_or(0)),
            NotOmf(_) => (Status::NotOmf, 0),
            NewerVersion(_, _) => (Status::NewerVersion, 0),
            PreReleaseVersion(_, _, _) => (Status::PreRelease, 0),
            DeserializationFailed(_) => (Status::DeserializationFailed, 0),
            SerializationFailed(_) => (Status::SerializationFailed, 0),
            ValidationFailed(_) => (Status::ValidationFailed, 0),
            LimitExceeded(_) => (Status::LimitExceeded, 0),
            NotImageData => (Status::NotImageData, 0),
            InvalidData(_) => (Status::InvalidData, 0),
            ImageError(_) => (Status::ImageError, 0),
            UnsafeCast(_, _) => (Status::UnsafeCast, 0),
            NotParquetData => (Status::NotParquetData, 0),
            ZipMemberMissing(_) => (Status::ZipMemberMissing, 0),
            ZipError(_) => (Status::ZipError, 0),
            ParquetSchemaMismatch(_, _) => (Status::ParquetSchemaMismatch, 0),
            ParquetError(_) => (Status::ParquetError, 0),
            _ => (Status::Panic, 0),
        };
        Self::External {
            code: status as i32,
            detail,
            message,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    #[default]
    Success = 0,
    Panic,
    InvalidArgument,
    InvalidCall,
    OutOfMemory,
    IoError,
    NotOmf,
    NewerVersion,
    PreRelease,
    DeserializationFailed,
    SerializationFailed,
    ValidationFailed,
    LimitExceeded,
    NotImageData,
    NotParquetData,
    ArrayTypeWrong,
    BufferLengthWrong,
    InvalidData,
    UnsafeCast,
    ZipMemberMissing,
    ZipError,
    ParquetSchemaMismatch,
    ParquetError,
    ImageError,
}

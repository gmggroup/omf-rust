//! Contains the [`Reader`] and [`Writer`] objects.

#[cfg(feature = "image")]
mod image;
#[cfg(feature = "parquet")]
pub(crate) mod parquet;
mod reader;
mod sub_file;
mod writer;
mod zip_container;

pub use reader::{Limits, Reader};
pub use sub_file::SubFile;
pub use writer::{Compression, Writer};

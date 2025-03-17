//! Contains the [`Reader`] and [`Writer`] objects.

#[cfg(feature = "image")]
mod image;
#[cfg(feature = "parquet")]
pub(crate) mod parquet;
mod read_at;
mod reader;
mod sub_file;
mod write_to;
mod writer;
mod zip_container;

pub use read_at::ReadAt;
pub use reader::{Limits, Reader};
pub use sub_file::SubFile;
pub use write_to::WriteTo;
pub use writer::{Compression, Writer};

pub(crate) mod state;
mod types;

pub use state::{set_error, set_panic};
pub use types::{Error, InvalidArg};

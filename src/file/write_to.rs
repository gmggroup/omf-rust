use std::io::{Seek, Write};

pub trait WriteTo: Write + Seek {}

impl<T: Write + Seek> WriteTo for T {}

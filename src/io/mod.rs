pub mod reader;
pub mod writer;
pub mod extractor;

use std::result;

type Result<T> = result::Result<T, ()>;


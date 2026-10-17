#![no_std]
#[cfg(test)]
extern crate std;

mod error;
mod header;
mod program;

pub use error::ElfError;
pub use header::{ElfHeader, parse_header};

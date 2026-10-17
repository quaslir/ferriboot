#![no_std]
#[cfg(test)]
extern crate std;

mod error;
mod header;
mod program;

pub use error::ElfError;
pub use header::{ElfHeader, parse_header};
pub use program::{ProgramHeader, parse_program_header};

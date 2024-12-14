pub mod bits;

use std::fmt;
use std::fmt::Debug;
use pyo3::prelude::*;

#[pymodule]
fn bit_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<bits::Bits>()?;
    Ok(())
}

#[derive(Debug)]
pub enum BitsError {
    Error(String),
    OutOfBounds(u64, u64),
    InvalidCharacter(char),
    InvalidLength(u64),
    HexDecodeError(hex::FromHexError),
}

impl fmt::Display for BitsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitsError::Error(s) => write!(f, "{}", s),
            BitsError::OutOfBounds(i, l) => write!(f, "Index {} out of bounds of length {}.", i, l),
            BitsError::InvalidCharacter(c) => write!(f, "Invalid character in binary string: {}", c),
            BitsError::InvalidLength(len) => write!(f, "Invalid length: {}", len),
            BitsError::HexDecodeError(e) => write!(f, "Hex decode error: {}", e),
        }
    }
}

impl std::error::Error for BitsError {}

impl From<hex::FromHexError> for BitsError {
    fn from(err: hex::FromHexError) -> BitsError {
        BitsError::HexDecodeError(err)
    }
}

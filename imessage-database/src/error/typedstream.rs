/*!
 Errors that can happen when parsing `typedstream` data. This module is for the new `typedstream` deserializer.
*/

use std::{
    array::TryFromSliceError,
    fmt::{Display, Formatter, Result},
    str::Utf8Error,
};

/// Errors that can happen when parsing `typedstream` data
#[derive(Debug)]
pub enum TypedStreamError {
    OutOfBounds(usize, usize),
    InvalidHeader,
    SliceError(TryFromSliceError),
    StringParseError(Utf8Error),
    InvalidArray,
    InvalidPointer(u8),
}

impl Display for TypedStreamError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            TypedStreamError::OutOfBounds(idx, len) => {
                write!(fmt, "Index {idx:x} is outside of range {len:x}!")
            }
            TypedStreamError::InvalidHeader => write!(fmt, "Invalid typedstream header!"),
            TypedStreamError::SliceError(why) => {
                write!(fmt, "Unable to slice source stream: {why}")
            }
            TypedStreamError::StringParseError(why) => write!(fmt, "Failed to parse string: {why}"),
            TypedStreamError::InvalidArray => write!(fmt, "Failed to parse array data"),
            TypedStreamError::InvalidPointer(why) => write!(fmt, "Failed to parse pointer: {why}"),
        }
    }
}

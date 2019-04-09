//! This crate contains all the tools we need to generate nice pdf documents.

#![warn(missing_docs)]

pub mod font;

use std::path::PathBuf;
use std::{fmt, result};

macro_rules! impl_from_error {
    ($type: ty, $variant: path, $from: ty) => {
        impl From<$from> for $type {
            fn from(e: $from) -> $type {
                $variant(e)
            }
        }
    }
}

/// The error type of the library.
#[derive(Debug)]
pub enum Error {
    /// Error while dealing with freetype.
    FreetypeError(freetype::Error),

    /// Error while dealing with printpdf.
    PrintpdfError(printpdf::errors::Error),

    /// The specified font was not found.
    FontNotFound(PathBuf),

    /// The specified font has no name or no style.
    FontWithoutName(PathBuf),
}

impl_from_error!(Error, Error::FreetypeError, freetype::Error);
impl_from_error!(Error, Error::PrintpdfError, printpdf::errors::Error);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::FreetypeError(e) => write!(fmt, "freetype error: {}", e),
            Error::PrintpdfError(e) => write!(fmt, "printpdf error: {}", e),
            Error::FontNotFound(path) => write!(fmt, "couldn't find font \"{}\"", path.display()),
            Error::FontWithoutName(path) => write!(fmt, "font has no name or style \"{}\"", path.display()),
        }
    }
}

/// The result type of the library.
type Result<T> = result::Result<T, Error>;

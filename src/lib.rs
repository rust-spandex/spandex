//! This crate contains all the tools we need to generate nice pdf documents.

#![warn(missing_docs)]

pub mod config;
pub mod document;
pub mod font;
pub mod parser;
pub mod typography;
pub mod units;
pub mod ligature;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{error, fmt, io, result};

use printpdf::Pt;

use crate::config::Config;
use crate::parser::parse;
use crate::parser::error::Errors;

macro_rules! impl_from_error {
    ($type: ty, $variant: path, $from: ty) => {
        impl From<$from> for $type {
            fn from(e: $from) -> $type {
                $variant(e)
            }
        }
    };
}

/// The error type of the library.
#[derive(Debug)]
pub enum Error {
    /// Cannot read current directory.
    CannotReadCurrentDir,

    /// No spandex.toml was found.
    NoConfigFile,

    /// Error while dealing with freetype.
    FreetypeError(freetype::Error),

    /// Error while dealing with printpdf.
    PrintpdfError(printpdf::errors::Error),

    /// The specified font was not found.
    FontNotFound(PathBuf),

    /// The specified font has no name or no style.
    FontWithoutName(PathBuf),

    /// An error occured while loading an hyphenation dictionnary.
    HyphenationLoadError(hyphenation::load::Error),

    /// Another io error occured.
    IoError(io::Error),

    /// Some error occured while parsing a dex file.
    DexError(Errors),
}

impl_from_error!(Error, Error::FreetypeError, freetype::Error);
impl_from_error!(Error, Error::PrintpdfError, printpdf::errors::Error);
impl_from_error!(Error, Error::IoError, io::Error);
impl_from_error!(Error, Error::HyphenationLoadError, hyphenation::load::Error);
impl_from_error!(Error, Error::DexError, Errors);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CannotReadCurrentDir => write!(fmt, "cannot read current directory"),
            Error::NoConfigFile => write!(fmt, "no spandex.toml was found"),
            Error::FreetypeError(e) => write!(fmt, "freetype error: {}", e),
            Error::PrintpdfError(e) => write!(fmt, "printpdf error: {}", e),
            Error::FontNotFound(path) => write!(fmt, "couldn't find font \"{}\"", path.display()),
            Error::FontWithoutName(path) => {
                write!(fmt, "font has no name or style \"{}\"", path.display())
            }
            Error::HyphenationLoadError(e) => write!(fmt, "Problem with hyphenation: {}", e),
            Error::IoError(e) => write!(fmt, "an io error occured: {}", e),
            Error::DexError(e) => write!(fmt, "{}", e),
        }
    }
}

impl error::Error for Error {}

/// The result type of the library.
pub type Result<T> = result::Result<T, Error>;

/// Compiles a spandex project.
pub fn build(config: &Config) -> Result<()> {
    let (mut document, font_manager) = config.init()?;
    let font_config = font_manager.default_config();

    let mut content = String::new();
    let mut file = File::open(&config.input)?;
    file.read_to_string(&mut content)?;

    if config.input.ends_with(".dex") {
        let parsed = parse(&config.input)?;
        println!("{}", parsed.warnings);
        println!("{:?}", parsed.ast);
        document.render(&parsed.ast, &font_config, Pt(10.0));
    } else {
        document.write_content(&content, &font_config, Pt(10.0));
    }
    document.save("output.pdf");
    Ok(())
}

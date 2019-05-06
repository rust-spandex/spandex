//! This crate contains all the tools we need to generate nice pdf documents.

#![warn(missing_docs)]

#[macro_use] extern crate log;
#[macro_use] extern crate nom;

pub mod config;
pub mod document;
pub mod font;
pub mod typography;
pub mod units;
pub mod parser;

use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::{error, fmt, io, result};

use crate::config::Config;
use crate::units::Pt;
use crate::parser::parse;

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
    DexError(parser::Errors),
}

impl_from_error!(Error, Error::FreetypeError, freetype::Error);
impl_from_error!(Error, Error::PrintpdfError, printpdf::errors::Error);
impl_from_error!(Error, Error::IoError, io::Error);
impl_from_error!(Error, Error::HyphenationLoadError, hyphenation::load::Error);
impl_from_error!(Error, Error::DexError, parser::Errors);

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

    let regular_font_name = "CMU Serif Roman";
    let bold_font_name = "CMU Serif Bold";
    let italic_font_name = "CMU Serif Italic";
    let bold_italic_font_name = "CMU Serif BoldItalic";

    let font_config = font_manager.config(
        regular_font_name,
        bold_font_name,
        italic_font_name,
        bold_italic_font_name)?;

    let mut content = String::new();
    let mut file = File::open(&config.input)?;
    file.read_to_string(&mut content)?;

    if config.input.ends_with(".md") || config.input.ends_with(".mdown") {
        document.write_markdown(&content, &font_config, Pt(10.0).into());
    } else if config.input.ends_with(".dex") {
        let parsed = parse(&config.input)?;
        document.render(&parsed.ast, &font_config, Pt(10.0).into());
    } else {
        document.write_content(&content, &font_config, Pt(10.0).into());
    }
    document.save("output.pdf");
    Ok(())
}

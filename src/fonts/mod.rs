//! This module contains everything that helps us dealing with fonts.

pub mod configuration;
pub mod manager;
pub mod styles;

use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use freetype::{face, Face, Library};
use printpdf::types::plugins::graphics::two_dimensional::font::IndirectFontRef;
use printpdf::Pt;

use crate::document::Document;
use crate::{Error, Result};

/// A font that contains the printpdf object font needed to render text and the freetype font
/// needed to measure text.
#[derive(Debug)]
pub struct Font {
    /// The freetype face.
    freetype: Face,

    /// The printpdf font.
    printpdf: IndirectFontRef,
}

impl Font {
    /// Creates a font from a path to a file.
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        library: &Library,
        document: &mut Document,
    ) -> Result<Font> {
        let file = File::open(path.as_ref())
            .map_err(|_| Error::FontNotFound(PathBuf::from(path.as_ref())))?;
        Ok(Font {
            freetype: library.new_face(path.as_ref(), 0)?,
            printpdf: document.inner_mut().add_external_font(file)?,
        })
    }

    /// Creates a font from a byte array.
    pub fn from_bytes(bytes: &[u8], library: &Library, document: &mut Document) -> Result<Font> {
        let cursor = Cursor::new(bytes);
        Ok(Font {
            // I don't like this bytes.to_vec() but I'm not sure there's a better way of doing
            // this...
            freetype: library.new_memory_face(bytes.to_vec(), 0)?,
            printpdf: document.inner_mut().add_external_font(cursor)?,
        })
    }

    /// Computes the width of a char of the font at a specified size.
    pub fn char_width(&self, c: char, scale: Pt) -> Pt {
        let scale = scale.0;

        // vertical scale for the space character
        let vert_scale = {
            if self
                .freetype
                .load_char(0x0020, face::LoadFlag::NO_SCALE)
                .is_ok()
            {
                self.freetype.glyph().metrics().vertAdvance
            } else {
                1000
            }
        };

        // calculate the width of the text in unscaled units
        let is_ok = self
            .freetype
            .load_char(c as usize, face::LoadFlag::NO_SCALE)
            .is_ok();

        let width = if is_ok {
            self.freetype.glyph().metrics().horiAdvance
        } else {
            0
        };

        Pt(width as f64 / (vert_scale as f64 / scale))
    }

    /// Computes the text width of the font at a specified size.
    pub fn text_width(&self, text: &str, scale: Pt) -> Pt {
        let scale = scale.0;

        // vertical scale for the space character
        let vert_scale = {
            if self
                .freetype
                .load_char(0x0020, face::LoadFlag::NO_SCALE)
                .is_ok()
            {
                self.freetype.glyph().metrics().vertAdvance
            } else {
                1000
            }
        };

        // calculate the width of the text in unscaled units
        let sum_width = text.chars().fold(0, |acc, ch| {
            let is_ok = self
                .freetype
                .load_char(ch as usize, face::LoadFlag::NO_SCALE)
                .is_ok();

            if is_ok {
                acc + self.freetype.glyph().metrics().horiAdvance
            } else {
                acc
            }
        });

        Pt(sum_width as f64 / (vert_scale as f64 / scale))
    }

    /// Returns a reference to the printpdf font.
    pub fn printpdf(&self) -> &IndirectFontRef {
        &self.printpdf
    }
}

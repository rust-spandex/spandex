//! This module holds all the mathematical logic around typesetting.

pub mod items;
pub mod paragraphs;
pub mod justification;

use crate::units::Sp;
use crate::font::Font;

/// A glyph with its font style.
#[derive(Clone)]
pub struct Glyph<'a> {
    /// The content of the word.
    pub glyph: char,

    /// The font style of the word.
    pub font: &'a Font,

    /// The size of the font.
    pub scale: Sp,
}

impl<'a> Glyph<'a> {
    /// Creates a new word from a string and a font style.
    pub fn new(glyph: char, font: &'a Font, scale: Sp) -> Glyph<'a> {
        Glyph {
            glyph,
            font,
            scale,
        }
    }
}

//! This module holds all the mathematical logic around typesetting.

pub mod items;
pub mod justification;
pub mod paragraphs;

use crate::font::Font;
use crate::units::Sp;

/// A glyph with its font style.
#[derive(Debug, Clone)]
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
        Glyph { glyph, font, scale }
    }
}

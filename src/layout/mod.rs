pub mod constants;
pub mod paragraphs;

use crate::fonts::Font;
use printpdf::Pt;

/// A glyph with its font style.
#[derive(Debug, Clone)]
pub struct Glyph<'a> {
    /// The content of the word.
    pub glyph: char,

    /// The font style of the word.
    pub font: &'a Font,

    /// The size of the font.
    pub scale: Pt,
}

impl<'a> Glyph<'a> {
    /// Creates a new word from a string and a font style.
    pub fn new(glyph: char, font: &'a Font, scale: Pt) -> Glyph<'a> {
        Glyph { glyph, font, scale }
    }
}

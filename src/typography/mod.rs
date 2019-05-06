//! This module holds all the mathematical logic around typesetting.

pub mod items;
pub mod paragraphs;
pub mod justification;

use crate::font::FontStyle;

/// A word with its font style.
#[derive(Clone)]
pub struct Word {
    /// The content of the word.
    pub string: String,

    /// The font style of the word.
    pub font_style: FontStyle,
}

impl Word {
    /// Creates a new word from a string and a font style.
    pub fn new(string: String, font_style: FontStyle) -> Word {
        Word {
            string,
            font_style,
        }
    }
}

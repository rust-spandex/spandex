use crate::fonts::styles::FontStyle;
use crate::fonts::Font;

/// A font configuration for a document.
pub struct FontConfig<'a> {
    /// The regular font.
    pub regular: &'a Font,

    /// The bold font.
    pub bold: &'a Font,

    /// The italic font.
    pub italic: &'a Font,

    /// The bold italic font.
    pub bold_italic: &'a Font,
}

impl<'a> FontConfig<'a> {
    /// Returns the font corresponding to the style.
    pub fn for_style(&self, style: FontStyle) -> &Font {
        match (style.bold, style.italic) {
            (false, false) => self.regular,
            (true, false) => self.bold,
            (false, true) => self.italic,
            (true, true) => self.bold_italic,
        }
    }
}

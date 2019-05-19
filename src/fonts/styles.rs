//! Different style variants of a `Font`.

/// A style for a font. It can be bold, italic, both or none.
#[derive(Copy, Clone, Debug)]
pub struct FontStyle {
    /// Whether the bold is activated or not.
    pub bold: bool,

    /// Whether the italic is activated or not.
    pub italic: bool,
}

impl FontStyle {
    /// Creates a new regular font style.
    pub fn regular() -> FontStyle {
        FontStyle {
            bold: false,
            italic: false,
        }
    }

    /// Adds the bold style to the font.
    pub fn bold(self) -> FontStyle {
        FontStyle {
            bold: true,
            italic: self.italic,
        }
    }

    /// Adds the italic style to the font.
    pub fn italic(self) -> FontStyle {
        FontStyle {
            bold: self.bold,
            italic: true,
        }
    }

    /// Removes the bold style from the font.
    pub fn unbold(self) -> FontStyle {
        FontStyle {
            bold: false,
            italic: self.italic,
        }
    }

    /// Removes the italic style from the font.
    pub fn unitalic(self) -> FontStyle {
        FontStyle {
            bold: self.bold,
            italic: false,
        }
    }
}

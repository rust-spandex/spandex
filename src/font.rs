//! This module contains everything that helps us dealing with fonts.

use std::io::Cursor;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use freetype::{Face, Library, face};

use printpdf::types::plugins::graphics::two_dimensional::font::IndirectFontRef;

use crate::{Error, Result};
use crate::units::{Pt, Sp};
use crate::document::Document;

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
    pub fn from_file<P: AsRef<Path>>(path: P, library: &Library, document: &mut Document) -> Result<Font> {
        let file = File::open(path.as_ref()).map_err(|_| Error::FontNotFound(PathBuf::from(path.as_ref())))?;
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
    pub fn char_width(&self, c: char, scale: Sp) -> Sp {
        let scale: Pt = scale.into();
        let scale = scale.0;

        // vertical scale for the space character
        let vert_scale = {
            if let Ok(_) = self.freetype.load_char(0x0020, face::LoadFlag::NO_SCALE) {
                self.freetype.glyph().metrics().vertAdvance
            } else {
                1000
            }
        };

        // calculate the width of the text in unscaled units
        let width = if let Ok(_) = self.freetype.load_char(c as usize, face::LoadFlag::NO_SCALE) {
            self.freetype.glyph().metrics().horiAdvance
        } else {
            0
        };

        Pt(width as f64 / (vert_scale as f64 / scale)).into()
    }

    /// Computes the text width of the font at a specified size.
    pub fn text_width(&self, text: &str, scale: Sp) -> Sp {

        let scale: Pt = scale.into();
        let scale = scale.0;

        // vertical scale for the space character
        let vert_scale = {
            if let Ok(_) = self.freetype.load_char(0x0020, face::LoadFlag::NO_SCALE) {
                self.freetype.glyph().metrics().vertAdvance
            } else {
                1000
            }
        };

        // calculate the width of the text in unscaled units
        let sum_width = text.chars().fold(0, |acc, ch| {
            if let Ok(_) = self.freetype.load_char(ch as usize, face::LoadFlag::NO_SCALE) {
                let glyph_w = self.freetype.glyph().metrics().horiAdvance;
                acc + glyph_w
            } else { acc }
        });

        Pt(sum_width as f64 / (vert_scale as f64 / scale)).into()
    }

    /// Returns a reference to the printpdf font.
    pub fn printpdf(&self) -> &IndirectFontRef {
        &self.printpdf
    }
}

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

/// This struct holds the different fonts.
pub struct FontManager {
    /// The freetype library, needed to be able to measure texts.
    library: Library,

    /// The hashmap that associates names of fonts with fonts.
    fonts: HashMap<String, Font>,
}

impl FontManager {
    /// Creates a new font manager, with the default fonts.
    pub fn init(document: &mut Document) -> Result<FontManager> {

        let mut font_manager = FontManager {
            library: Library::init()?,
            fonts: HashMap::new(),
        };

        // Insert the default fonts
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbl.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbmo.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbmr.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbso.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbsr.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbtl.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbto.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunbx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunci.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunit.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunobi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunobx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunorm.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunoti.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunrm.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunsi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunsl.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunso.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunssdc.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunss.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunsx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmuntb.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunti.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmuntt.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmuntx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunui.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunvi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../assets/fonts/cmunvt.ttf"), document)?;

        Ok(font_manager)

    }

    /// Adds a new font to the font manager.
    pub fn add_font(&mut self, bytes: &[u8], document: &mut Document) -> Result<()> {
        let font = Font::from_bytes(bytes, &self.library, document)?;
        let name = match (font.freetype.family_name(), font.freetype.style_name()) {
            (Some(family), Some(style)) => format!("{} {}", family, style),
            _ => {
                error!("Failed to create a built in font, this is a implementation error");
                unreachable!();
            },
        };
        self.fonts.insert(name, font);
        Ok(())
    }

    /// Returns a reference font if it is present in the font manager.
    pub fn get(&self, font_name: &str) -> Option<&Font> {
        self.fonts.get(font_name)
    }

    /// Creates a font config.
    pub fn config<'a>(&'a self, regular: &str, bold: &str, italic: &str, bold_italic: &str)
        -> Result<FontConfig<'a>> {

        Ok(FontConfig {
            regular: self.fonts.get(regular).ok_or(Error::FontNotFound(PathBuf::from(regular)))?,
            bold: self.fonts.get(bold).ok_or(Error::FontNotFound(PathBuf::from(bold)))?,
            italic: self.fonts.get(italic).ok_or(Error::FontNotFound(PathBuf::from(italic)))?,
            bold_italic: self.fonts.get(bold_italic).ok_or(Error::FontNotFound(PathBuf::from(bold_italic)))?,
        })
    }

    /// Returns the default configuration for computer modern fonts.
    pub fn default_config<'a>(&'a self) -> FontConfig<'a> {
        let regular = "CMU Serif Roman";
        let bold = "CMU Serif Bold";
        let italic = "CMU Serif Italic";
        let bold_italic = "CMU Serif BoldItalic";

        // This should never fail.
        match self.config(regular, bold, italic, bold_italic) {
            Ok(c) => c,
            Err(_) => unreachable!("Default font not found, this should never happen"),
        }
    }

}

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

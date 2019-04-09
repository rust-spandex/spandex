//! This module contains everything that helps us dealing with fonts.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use freetype::{Face, Library, face};

use printpdf::types::plugins::graphics::two_dimensional::font::IndirectFontRef;

use crate::{Error, Result};
use crate::document::Document;

/// A font that contains the printpdf object font needed to render text and the freetype font
/// needed to measure text.
pub struct Font {
    /// The freetype face.
    freetype: Face,

    /// The printpdf font.
    printpdf: IndirectFontRef,
}

impl Font {
    /// Creates a font from a file.
    pub fn create<P: AsRef<Path>>(path: P, library: &Library, document: &mut Document) -> Result<Font> {
        let file = File::open(path.as_ref()).map_err(|_| Error::FontNotFound(PathBuf::from(path.as_ref())))?;
        Ok(Font {
            freetype: library.new_face(path.as_ref(), 0)?,
            printpdf: document.inner_mut().add_external_font(file)?,
        })
    }
}

/// The different fonts that will be used in a document.
pub enum FontType {
    /// The regular font.
    Regular,
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

        // Insert the default fonts.
        font_manager.add_font("assets/fonts/cmunbi.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbl.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbmo.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbmr.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbso.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbsr.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbtl.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbto.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunbx.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunci.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunit.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunobi.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunobx.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunorm.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunoti.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunrm.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunsi.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunsl.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunso.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunssdc.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunss.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunsx.ttf", document)?;
        font_manager.add_font("assets/fonts/cmuntb.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunti.ttf", document)?;
        font_manager.add_font("assets/fonts/cmuntt.ttf", document)?;
        font_manager.add_font("assets/fonts/cmuntx.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunui.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunvi.ttf", document)?;
        font_manager.add_font("assets/fonts/cmunvt.ttf", document)?;

        Ok(font_manager)

    }

    /// Adds a new font to the font manager.
    pub fn add_font<P: AsRef<Path>>(&mut self, path: P, document: &mut Document) -> Result<()> {
        let font = Font::create(&path, &self.library, document)?;
        let name = match (font.freetype.family_name(), font.freetype.style_name()) {
            (Some(family), Some(style)) => format!("{} {}", family, style),
            _ => return Err(Error::FontWithoutName(PathBuf::from(path.as_ref()))),
        };
        self.fonts.insert(name, font);
        Ok(())
    }

    /// Returns a reference font if it is present in the font manager.
    pub fn get(&self, font_name: &str) -> Option<&Font> {
        self.fonts.get(font_name)
    }

    /// Computes the text width of a font.
    pub fn text_width(&self, font: &Font, scale: i64, text: &str) -> f64 {
        // vertical scale for the space character
        let vert_scale = {
            if let Ok(_) = font.freetype.load_char(0x0020, face::LoadFlag::NO_SCALE) {
                font.freetype.glyph().metrics().vertAdvance
            } else {
                1000
            }
        };

        // calculate the width of the text in unscaled units
        let sum_width = text.chars().fold(0, |acc, ch| {
            if let Ok(_) = font.freetype.load_char(ch as usize, face::LoadFlag::NO_SCALE) {
                let glyph_w = font.freetype.glyph().metrics().horiAdvance;
                acc + glyph_w
            } else { acc }
        });

        sum_width as f64 / (vert_scale as f64 / scale as f64)
    }

}

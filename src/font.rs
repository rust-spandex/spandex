//! This module contains everything that helps us dealing with fonts.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use freetype::{Face, Library};

use printpdf::PdfDocumentReference;
use printpdf::types::plugins::graphics::two_dimensional::font::IndirectFontRef;

use crate::{Error, Result};

/// A font that contains the printpdf object font needed to render text and the freetype font
/// needed to measure text.
pub struct Font {
    /// The freetype face.
    freetype: Face,

    /// The printpdf font.
    printpdf: IndirectFontRef,
}

impl Font {
    pub fn create<P: AsRef<Path>>(path: P, library: &Library, document: &PdfDocumentReference) -> Result<Font> {
        let file = File::open(path.as_ref()).map_err(|_| Error::FontNotFound(PathBuf::from(path.as_ref())))?;
        Font {
            freetype: library.new_face(path.as_ref(), 0)?,
            printpdf: document.add_external_font(file)?,
        };

        panic!();
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
    pub fn init(document: &mut PdfDocumentReference) -> Result<FontManager> {

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
    pub fn add_font<P: AsRef<Path>>(&mut self, path: P, document: &mut PdfDocumentReference) -> Result<()> {
        let font = Font::create(&path, &self.library, document)?;
        let name = match (font.freetype.family_name(), font.freetype.style_name()) {
            (Some(family), Some(style)) => format!("{} {}", family, style),
            _ => return Err(Error::FontWithoutName(PathBuf::from(path.as_ref()))),
        };
        self.fonts.insert(name, font);
        Ok(())
    }

}

use crate::document::Document;
use crate::fonts::configuration::FontConfig;
use crate::fonts::Font;
use crate::{Error, Result};
use freetype::Library;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbl.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbmo.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbmr.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbso.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbsr.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbtl.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbto.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunbx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunci.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunit.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunobi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunobx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunorm.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunoti.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunrm.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunsi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunsl.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunso.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunssdc.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunss.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunsx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmuntb.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunti.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmuntt.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmuntx.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunui.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunvi.ttf"), document)?;
        font_manager.add_font(include_bytes!("../../assets/fonts/cmunvt.ttf"), document)?;

        Ok(font_manager)
    }

    /// Adds a new font to the font manager.
    pub fn add_font(&mut self, bytes: &[u8], document: &mut Document) -> Result<()> {
        let font = Font::from_bytes(bytes, &self.library, document)?;
        let name = match (font.freetype.family_name(), font.freetype.style_name()) {
            (Some(family), Some(style)) => format!("{} {}", family, style),
            _ => {
                eprintln!("Failed to create a built in font, this is a implementation error");
                unreachable!();
            }
        };
        self.fonts.insert(name, font);
        Ok(())
    }

    /// Returns a reference font if it is present in the font manager.
    pub fn get(&self, font_name: &str) -> Option<&Font> {
        self.fonts.get(font_name)
    }

    /// Creates a font config.
    pub fn config<'a>(
        &'a self,
        regular: &str,
        bold: &str,
        italic: &str,
        bold_italic: &str,
    ) -> Result<FontConfig<'a>> {
        Ok(FontConfig {
            regular: self
                .fonts
                .get(regular)
                .ok_or_else(|| Error::FontNotFound(PathBuf::from(regular)))?,
            bold: self
                .fonts
                .get(bold)
                .ok_or_else(|| Error::FontNotFound(PathBuf::from(bold)))?,
            italic: self
                .fonts
                .get(italic)
                .ok_or_else(|| Error::FontNotFound(PathBuf::from(italic)))?,
            bold_italic: self
                .fonts
                .get(bold_italic)
                .ok_or_else(|| Error::FontNotFound(PathBuf::from(bold_italic)))?,
        })
    }

    /// Returns the default configuration for computer modern fonts.
    pub fn default_config(&self) -> FontConfig {
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

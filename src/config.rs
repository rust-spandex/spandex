//! This module contains the configuration struct for the spandex.toml file.

use serde::{Serialize, Deserialize};

use crate::units::{Sp, Mm};
use crate::document::Window;
use crate::document::Document;
use crate::font::FontManager;
use crate::Result;

/// This structure holds all the configuration information.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// The title of the document.
    pub title: String,

    /// The page size of the document.
    pub page_size: (Sp, Sp),

    /// The top margin of the document.
    pub top_margin: Sp,

    /// The left margin of the document.
    pub left_margin: Sp,

    /// The text width of the document.
    pub text_width: Sp,

    /// The text height of the document.
    pub text_height: Sp,

    /// The path to the first file of the spandex content.
    pub input: String,
}

impl Config {
    /// Creates a default configuration with a title.
    pub fn with_title(title: &str) -> Config {
        Config {
            title: String::from(title),
            page_size: (Sp::from(Mm(210.0)), Sp::from(Mm(297.0))),
            top_margin: Sp::from(Mm(30.0)),
            left_margin: Sp::from(Mm(30.0)),
            text_width: Sp::from(Mm(150.0)),
            text_height: Sp::from(Mm(237.0)),
            input: String::from("main.dex"),
        }
    }

    /// Creates a document and a font maanger from the config.
    pub fn init(&self) -> Result<(Document, FontManager)> {

        let window = Window {
            x: self.left_margin,
            y: self.top_margin,
            width: self.text_width,
            height: self.text_height,
        };

        let mut document = Document::new("Hello", self.page_size.0, self.page_size.1, window);
        let font_manager = FontManager::init(&mut document)?;

        Ok((document, font_manager))

    }

}

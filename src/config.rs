//! This module contains the configuration struct for the spandex.toml file.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::units::{Sp, Pt, Mm};
use crate::document::Window;
use crate::document::{Document, pt};
use crate::font::FontManager;
use crate::{Error, Result};

/// This structure holds all the configuration information.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// The title of the document.
    pub title: String,

    /// The page size of the document.
    pub page_size: (i64, i64),

    /// The top margin of the document.
    pub top_margin: i64,

    /// The left margin of the document.
    pub left_margin: i64,

    /// The text width of the document.
    pub text_width: i64,

    /// The text height of the document.
    pub text_height: i64,

    /// The path to the first file of the spandex content.
    pub input: String,
}

impl Config {
    /// Creates a default configuration with a title.
    pub fn with_title(title: &str) -> Config {
        Config {
            title: String::from(title),
            page_size: (Sp::from(Mm(210.0)).0, Sp::from(Mm(297.0)).0),
            top_margin: Sp::from(Mm(30.0)).0,
            left_margin: Sp::from(Mm(30.0)).0,
            text_width: Sp::from(Mm(150.0)).0,
            text_height: Sp::from(Mm(237.0)).0,
            input: String::from("main.txt"),
        }
    }

    /// Triggers the build of the document.
    pub fn build(&self) -> Result<()> {
        let page_width: Pt = Sp(self.page_size.0).into();
        let page_height: Pt = Sp(self.page_size.1).into();

        let text_width: Pt = Sp(self.text_width).into();
        let text_height: Pt = Sp(self.text_height).into();

        let left_margin: Pt = Sp(self.left_margin).into();
        let top_margin: Pt = Sp(self.top_margin).into();

        let window = Window {
            x: left_margin.0,
            y: top_margin.0,
            width: text_width.0,
            height: text_height.0,
        };

        let mut document = Document::new("Hello", page_width.0, page_height.0, window);

        let font_manager = FontManager::init(&mut document)?;
        let font_name = "CMU Serif Roman";

        let font = font_manager.get(font_name)
            .ok_or(Error::FontNotFound(PathBuf::from(font_name)))?;

        let mut content = String::new();
        let mut file = File::open(&self.input)?;
        file.read_to_string(&mut content)?;
        document.write_content(&content, font, 10.0);
        document.save("output.pdf");
        Ok(())
    }

}

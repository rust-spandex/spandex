//! This module contains the configuration struct for the spandex.toml file.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::units::{Sp, Pt, Mm};
use crate::document::Window;
use crate::document::Document;
use crate::font::FontManager;
use crate::{Error, Result};

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
            input: String::from("main.md"),
        }
    }

    /// Creates a document and a font maanger from the config.
    pub fn init(&self) -> Result<(Document, FontManager)> {

        let page_width: Pt = self.page_size.0.into();
        let page_height: Pt = self.page_size.1.into();

        let text_width: Pt = self.text_width.into();
        let text_height: Pt = self.text_height.into();

        let left_margin: Pt = self.left_margin.into();
        let top_margin: Pt = self.top_margin.into();

        let window = Window {
            x: left_margin.0,
            y: top_margin.0,
            width: text_width.0,
            height: text_height.0,
        };

        let mut document = Document::new("Hello", page_width.0, page_height.0, window);
        let font_manager = FontManager::init(&mut document)?;

        Ok((document, font_manager))

    }

    /// Triggers the build of the document.
    pub fn build(&self) -> Result<()> {

        let (mut document, font_manager) = self.init()?;

        let regular_font_name = "CMU Bright Roman";
        let bold_font_name = "CMU Serif Bold";

        let font_config = font_manager.config(regular_font_name, bold_font_name)?;

        let font = font_manager.get(regular_font_name)
            .ok_or(Error::FontNotFound(PathBuf::from(regular_font_name)))?;

        let mut content = String::new();
        let mut file = File::open(&self.input)?;
        file.read_to_string(&mut content)?;

        if self.input.ends_with(".md") || self.input.ends_with(".mdown") {
            document.write_markdown(&content, &font_config, 10.0);
        } else {
            document.write_content(&content, font, 10.0);
        }
        document.save("output.pdf");
        Ok(())
    }

}

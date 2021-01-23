//! This module defines the basic configuration of a document that is to be
//! typeset. The configuration is parsed from a TOML file located at the
//! root of the SpanDeX project. Mandatory measurements take default values
//! that are also provided by this module.

use std::{fmt, result};

use printpdf::{Mm, Pt};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::document::{Document, Window};
use crate::fonts::manager::FontManager;
use crate::Result as CResult;

/// Serializes a `Pt` structure.
// This is required to use in macro `serialize_with`.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_pt<S: Serializer>(pt: &Pt, serializer: S) -> result::Result<S::Ok, S::Error> {
    serializer.serialize_f64(pt.0)
}

/// Deserializes a `Pt` structure.
pub fn deserialize_pt<'a, D: Deserializer<'a>>(deserializer: D) -> Result<Pt, D::Error> {
    deserializer.deserialize_f64(PtVisitor)
}

macro_rules! visit_from {
    ($visit: ident, $ty: ty) => {
        fn $visit<E>(self, value: $ty) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Pt(f64::from(value)))
        }
    };
}

macro_rules! visit_as {
    ($visit: ident, $ty: ty) => {
        fn $visit<E>(self, value: $ty) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Pt(value as f64))
        }
    };
}

/// Visitor for the `Pt` structure.
pub struct PtVisitor;

impl<'a> Visitor<'a> for PtVisitor {
    type Value = Pt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a floatting point number")
    }

    visit_from!(visit_u8, u8);
    visit_from!(visit_u16, u16);
    visit_from!(visit_u32, u32);
    visit_as!(visit_u64, u64);
    visit_from!(visit_i8, i8);
    visit_from!(visit_i16, i16);
    visit_from!(visit_i32, i32);
    visit_as!(visit_i64, i64);
    visit_from!(visit_f32, f32);
    visit_from!(visit_f64, f64);
}

/// Holds the configuration of a document, including various measurements
/// common to all pages.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// The title of the document.
    pub title: String,

    /// The width of the page of the document.
    #[serde(serialize_with = "serialize_pt")]
    #[serde(deserialize_with = "deserialize_pt")]
    pub page_width: Pt,

    /// The height of the page of the document.
    #[serde(serialize_with = "serialize_pt")]
    #[serde(deserialize_with = "deserialize_pt")]
    pub page_height: Pt,

    /// The top margin of the document.
    #[serde(serialize_with = "serialize_pt")]
    #[serde(deserialize_with = "deserialize_pt")]
    pub top_margin: Pt,

    /// The left margin of the document.
    #[serde(serialize_with = "serialize_pt")]
    #[serde(deserialize_with = "deserialize_pt")]
    pub left_margin: Pt,

    /// The text width of the document.
    #[serde(serialize_with = "serialize_pt")]
    #[serde(deserialize_with = "deserialize_pt")]
    pub text_width: Pt,

    /// The text height of the document.
    #[serde(serialize_with = "serialize_pt")]
    #[serde(deserialize_with = "deserialize_pt")]
    pub text_height: Pt,

    /// The path to the first file of the spandex content.
    pub input: String,
}

impl Config {
    /// Creates a default configuration with a title.
    pub fn with_title(title: &str) -> Config {
        let page_width: Pt = Mm(210.0).into();
        let page_height: Pt = Mm(297.0).into();
        let top_margin: Pt = Mm(30.0).into();
        let left_margin: Pt = Mm(30.0).into();
        let text_width: Pt = Mm(150.0).into();
        let text_height: Pt = Mm(237.0).into();

        Config {
            title: String::from(title),
            page_width,
            page_height,
            top_margin,
            left_margin,
            text_width,
            text_height,
            input: String::from("main.dex"),
        }
    }

    /// Creates a document and a font maanger from the config.
    pub fn init(&self) -> CResult<(Document, FontManager)> {
        let window = Window {
            x: self.left_margin,
            y: self.top_margin,
            width: self.text_width,
            height: self.text_height,
        };

        let mut document = Document::new("Hello", self.page_width, self.page_height, window);
        let font_manager = FontManager::init(&mut document)?;

        Ok((document, font_manager))
    }
}

use crate::document::{Document, Window};
use crate::font::FontManager;
use crate::Result as CResult;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, result};

use printpdf::{Mm, Pt};

pub fn serialize_pt<S: Serializer>(pt: &Pt, serializer: S) -> result::Result<S::Ok, S::Error> {
    serializer.serialize_f64(pt.0)
}

macro_rules! visit {
    ($visit: ident, $ty: ty) => {
        fn $visit<E>(self, value: $ty) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(Pt(value as f64))
        }
    }
}

pub struct PtVisitor;

impl<'a> Visitor<'a> for PtVisitor {
    type Value = Pt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a floatting point number")
    }

    visit!(visit_u8, u8);
    visit!(visit_u16, u16);
    visit!(visit_u32, u32);
    visit!(visit_u64, u64);
    visit!(visit_i8, i8);
    visit!(visit_i16, i16);
    visit!(visit_i32, i32);
    visit!(visit_i64, i64);
    visit!(visit_f32, f32);
    visit!(visit_f64, f64);
}

pub fn deserialize_pt<'a, D: Deserializer<'a>>(deserializer: D) -> Result<Pt, D::Error> {
    deserializer.deserialize_f64(PtVisitor)
}

#[cfg(test)]
mod test {

    use crate::config::{deserialize_pt, serialize_pt};
    use printpdf::Pt;
    use rand::Rng;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct Test {
        #[serde(serialize_with = "serialize_pt")]
        #[serde(deserialize_with = "deserialize_pt")]
        pub value: Pt,
    }

    impl Test {
        pub fn as_f64(&self) -> f64 {
            self.value.0
        }
    }

    #[test]
    fn test() {
        for _ in 0..1000 {
            let num = rand::thread_rng().gen_range(-10000.0, 10000.0);
            let value = Test { value: Pt(num) };
            let encoded: Vec<u8> = bincode::serialize(&value).unwrap();
            let decoded: Test = bincode::deserialize(&encoded[..]).unwrap();
            assert_eq!(value.as_f64(), decoded.as_f64());
        }
    }
}

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

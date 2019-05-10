//! This module allows to create beautiful documents.

use std::fmt;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use printpdf::{PdfDocument, PdfDocumentReference, PdfPageReference, PdfLayerReference};

use hyphenation::load::Load;
use hyphenation::{Standard, Language};

use crate::typography::justification::{Justifier, NaiveJustifier};
use crate::font::{Font, FontConfig};
use crate::units::{Pt, Sp};
use crate::parser::Ast;
use crate::typography::paragraphs::itemize_ast;

/// The struct that manages the counters for the document.
#[derive(Copy, Clone)]
pub struct Counters {
    /// The section counter.
    pub sections: usize,

    /// The subsection couter.
    pub subsections: usize,

    /// The subsubsection counter.
    pub subsubsections: usize,
}

impl Counters {
    /// Creates a new empty counters.
    pub fn new() -> Counters {
        Counters {
            sections: 0,
            subsections: 0,
            subsubsections: 0,
        }
    }

    /// Increases the corresponding counter and returns it if it is correct.
    ///
    /// The counters of the subsections will be reinitialized.
    ///
    /// # Example
    ///
    /// ```
    /// # use spandex::document::Counters;
    /// let mut counters = Counters::new();
    /// counters.increment(1);
    /// assert_eq!(counters.sections, 1);
    /// assert_eq!(counters.subsections, 0);
    /// assert_eq!(counters.subsubsections, 0);
    /// counters.increment(2);
    /// assert_eq!(counters.subsections, 1);
    /// counters.increment(2);
    /// assert_eq!(counters.subsections, 2);
    /// counters.increment(1);
    /// assert_eq!(counters.sections, 2);
    /// assert_eq!(counters.subsections, 0);
    /// ```
    pub fn increment(&mut self, counter_id: i32) -> Option<usize> {
        match counter_id {
            1 => {
                self.sections += 1;
                self.subsections = 0;
                self.subsubsections = 0;
                Some(self.sections)
            }

            2 => {
                self.subsections += 1;
                self.subsubsections = 0;
                Some(self.subsections)
            },

            3 => {
                self.subsubsections += 1;
                Some(self.subsubsections)
            },

            _ => {
                warn!("sub sub sub sections are not supported");
                None
            },
        }
    }
}

impl fmt::Display for Counters {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.sections)?;
        if self.subsections > 0 {
            write!(fmt, ".{}", self.subsections)?;
        }
        if self.subsubsections > 0 {
            write!(fmt, ".{}", self.subsubsections)?;
        }
        Ok(())
    }
}

/// The window that is the part of the page on which we're allowed to write.
#[derive(Copy, Clone)]
pub struct Window {
    /// The x coordinate of the window, in pt.
    pub x: Sp,

    /// The y coordinate of the window, in pt.
    pub y: Sp,

    /// The width of the window, in pt.
    pub width: Sp,

    /// The height of the window, in pt.
    pub height: Sp,
}

/// This struct contains the pdf document.
pub struct Document {

    /// The inner document from printpdf.
    document: PdfDocumentReference,

    /// The current page.
    page: PdfPageReference,

    /// The current layer.
    layer: PdfLayerReference,

    /// The window on which we're allowed to write on the page.
    window: Window,

    /// The cursor, the position where we supposed to write next.
    cursor: (Sp, Sp),

    /// The current page size, in pt.
    page_size: (Sp, Sp),

    /// The counters of the document
    counters: Counters,
}

impl Document {
    /// Creates a new pdf document from its name and its size in pt.
    pub fn new<T: Into<Sp>, U: Into<Sp>>(name: &str, width: T, height: U, window: Window) -> Document {

        let width: Sp = width.into();
        let height: Sp = height.into();

        let (document, page, layer) = PdfDocument::new(name, width.into(), height.into(), "");

        let page = document.get_page(page);
        let layer = page.get_layer(layer);

        Document {
            document,
            page,
            layer,
            window,
            cursor: (window.x, window.height + window.y),
            page_size: (width.into(), height.into()),
            counters: Counters::new(),
        }

    }

    /// Returns a reference to the inner pdf document.
    pub fn inner(&self) -> &PdfDocumentReference {
        &self.document
    }

    /// Returns a mutable reference to the inner pdf document.
    pub fn inner_mut(&mut self) -> &mut PdfDocumentReference {
        &mut self.document
    }

    /// Renders an AST to the document.
    pub fn render(&mut self, ast: &Ast, font_config: &FontConfig, size: Sp) {

        let en = Standard::from_embedded(Language::EnglishUS).unwrap();

        match ast {
            Ast::Group(children) => {
                for child in children {
                    self.render(child, font_config, size);
                }
            },

            Ast::Title { .. } => {
                self.write_paragraph::<NaiveJustifier>(ast, font_config, size, &en);
                self.new_line(size);
            },

            Ast::Paragraph(_) => {
                self.write_paragraph::<NaiveJustifier>(ast, font_config, size, &en);
                self.new_line(size);
                self.new_line(size);
            },

            _ => (),
        }

    }

    /// Writes content on the document.
    pub fn write_content(&mut self, content: &str, font_config: &FontConfig, size: Sp) {

        let en = Standard::from_embedded(Language::EnglishUS).unwrap();

        for paragraph in content.split("\n") {

            // Ugly copy right there...
            let ast = Ast::Text(paragraph.to_owned());
            self.write_paragraph::<NaiveJustifier>(&ast, font_config, size, &en);
            self.new_line(size);
        }
    }

    /// Writes a paragraph on the document.
    pub fn write_paragraph<'a, J: Justifier>(&mut self, paragraph: &Ast, font_config: &FontConfig, size: Sp, dict: &Standard) {

        let paragraph = itemize_ast(paragraph, font_config, size, dict);
        let justified = J::justify(&paragraph, self.window.width);

        for line in justified {
            for glyph in line {

                self.layer.use_text(
                    glyph.0.glyph.to_string(),
                    Into::<Pt>::into(glyph.0.scale).0 as i64,
                    (self.window.x + glyph.1).into(),
                    self.cursor.1.into(),
                    glyph.0.font.printpdf());

            }

            self.new_line(size);
            self.cursor.0 = self.window.x;

            if self.cursor.1 <= size + self.window.y {
                self.new_page();
            }
        }
    }

    /// Writes a line in the document.
    pub fn write_line(&mut self, words: &[&str], font: &Font, size: Sp, spacing: Sp) {

        let size_i64 = Into::<Pt>::into(size).0 as i64;
        let mut current_width = self.window.x;

        for word in words {
            let width = current_width;
            let height = self.cursor.1;

            self.layer.use_text(word.to_owned(), size_i64, width.into(), (height - size).into(), font.printpdf());
            current_width += font.text_width(word, size) + spacing;
        }

        self.new_line(size);
    }

    /// Goes to the beginning of the next line.
    pub fn new_line(&mut self, size: Sp) {
        self.cursor.1 -= size;
    }

    /// Creates a new page and append it to the document.
    pub fn new_page(&mut self) {
        let page = self.document.add_page(self.page_size.0.into(), self.page_size.1.into(), "");
        self.page = self.document.get_page(page.0);
        self.layer = self.page.get_layer(page.1);
        self.cursor.1 = self.window.height + self.window.y;
    }

    /// Saves the document into a file.
    pub fn save<P: AsRef<Path>>(self, path: P) {
        let file = File::create(path.as_ref()).unwrap();
        let mut writer = BufWriter::new(file);
        self.document.save(&mut writer).unwrap();
    }
}

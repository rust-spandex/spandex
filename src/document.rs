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
use crate::parser::ast::Ast;
use crate::typography::paragraphs::itemize_ast;

/// The struct that manages the counters for the document.
#[derive(Clone)]
pub struct Counters {
    /// The counters.
    pub counters: Vec<usize>,
}

impl Counters {
    /// Creates a new empty counters.
    pub fn new() -> Counters {
        Counters {
            counters: vec![0],
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
    /// counters.increment(0);
    /// assert_eq!(counters.counter(0), 1);
    /// assert_eq!(counters.counter(1), 0);
    /// assert_eq!(counters.counter(2), 0);
    /// counters.increment(1);
    /// assert_eq!(counters.counter(1), 1);
    /// counters.increment(1);
    /// assert_eq!(counters.counter(1), 2);
    /// counters.increment(0);
    /// assert_eq!(counters.counter(0), 2);
    /// assert_eq!(counters.counter(1), 0);
    /// println!("{}", counters);
    /// ```
    pub fn increment(&mut self, counter_id: usize) -> usize {
        self.counters.resize(counter_id + 1, 0);
        self.counters[counter_id] += 1;
        self.counters[counter_id]
    }

    /// Returns a specific value of a counter.
    pub fn counter(&self, counter_id: usize) -> usize {
        match self.counters.get(counter_id) {
            Some(i) => *i,
            None => 0,
        }
    }
}

impl fmt::Display for Counters {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.counters.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("."))
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

            Ast::Title { level, content } => {
                self.counters.increment(*level as usize);
                match &**content {
                    Ast::Group(children) => {
                        let mut new_children = vec![Ast::Text(format!("{}", self.counters))];
                        new_children.extend_from_slice(children);
                        let new_ast = Ast::Title {
                            level: *level,
                            content: Box::new(Ast::Group(new_children)),
                        };
                        self.write_paragraph::<NaiveJustifier>(&new_ast, font_config, size, &en);
                    },
                    _ => self.write_paragraph::<NaiveJustifier>(ast, font_config, size, &en),
                }
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
            let ast = Ast::Text(paragraph.to_owned());
            self.write_paragraph::<NaiveJustifier>(&ast, font_config, size, &en);
            self.new_line(size);
        }
    }

    /// Writes a paragraph on the document.
    pub fn write_paragraph<'a, J: Justifier>(&mut self, paragraph: &Ast, font_config: &FontConfig, size: Sp, dict: &Standard) {

        let paragraph = itemize_ast(paragraph, font_config, size, dict, Sp(0));
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

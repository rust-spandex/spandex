//! This module allows to create beautiful documents.

use std::fmt;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use printpdf::{PdfDocument, PdfDocumentReference, PdfPageReference, PdfLayerReference, Mm, Pt};

use pulldown_cmark::{Event, Tag, Parser};

use crate::font::{Font, FontConfig};

/// Converts Pts to Mms.
pub fn mm(pts: f64) -> Mm {
    Into::<Mm>::into(Pt(pts as f64))
}

/// Converts Mms to Pts.
pub fn pt(mms: f64) -> f64 {
    Into::<Pt>::into(Mm(mms)).0
}

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
    pub x: f64,

    /// The y coordinate of the window, in pt.
    pub y: f64,

    /// The width of the window, in pt.
    pub width: f64,

    /// The height of the window, in pt.
    pub height: f64,
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
    cursor: (f64, f64),

    /// The current page size, in pt.
    page_size: (f64, f64),

    /// The counters of the document
    counters: Counters,
}

impl Document {
    /// Creates a new pdf document from its name and its size in pt.
    pub fn new(name: &str, width: f64, height: f64, window: Window) -> Document {

        let (document, page, layer) = PdfDocument::new(name, mm(width), mm(height), "");

        let page = document.get_page(page);
        let layer = page.get_layer(layer);

        Document {
            document,
            page,
            layer,
            window,
            cursor: (window.x, window.height + window.y),
            page_size: (width, height),
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

    /// Writes markdown content on the document.
    pub fn write_markdown(&mut self, markdown: &str, font_config: &FontConfig, size: f64) {

        let mut current_size = size;
        let mut content = String::new();

        let parser = Parser::new(markdown);

        for event in parser {
            match event {
                Event::Start(Tag::Header(i)) => {
                    if self.counters.increment(i).is_some() {
                        content.push_str(&format!("{}", self.counters));
                    }

                    current_size = size + 3.0 * (4 - i) as f64;
                },

                Event::Start(Tag::Item) => {
                    content.push_str(" - ");
                },

                Event::Text(ref text) => {
                    content.push(' ');
                    content.push_str(text);
                },

                Event::End(Tag::Paragraph) | Event::End(Tag::Item) => {
                    self.write_paragraph(&content, font_config.regular, current_size);
                    self.new_line(current_size);

                    content.clear();
                    current_size = size;
                },

                Event::End(Tag::Header(_)) => {
                    self.write_paragraph(&content, font_config.bold, current_size);
                    self.new_line(current_size);

                    content.clear();
                    current_size = size;
                },

                _ => (),
            }
            trace!("{:?}", event);
        }
    }

    /// Writes content on the document.
    pub fn write_content(&mut self, content: &str, font: &Font, size: f64) {
        for paragraph in content.split("\n") {
            self.write_paragraph(paragraph, font, size);
            self.new_line(size);
        }
    }

    /// Writes a paragraph on the document.
    pub fn write_paragraph(&mut self, paragraph: &str, font: &Font, size: f64) {
        debug!("{}", paragraph);
        let mut words = vec![];

        for word in paragraph.split_whitespace() {
            words.push(word);

            let line = words.join(" ");

            let text_width = font.text_width(&line, size);

            if text_width >= self.window.width {

                let remaining = words.pop().unwrap();
                let remaining_width = self.window.width - font.text_width(&words.join(" "), size);
                self.write_line(&words, font, size, 3.2 + remaining_width / (words.len() as f64));

                words.clear();
                words.push(remaining);

                if self.cursor.1 <= size + self.window.y {
                    self.new_page();
                }

            }
        }

        if ! words.is_empty() {
            self.write_line(&words, font, size, 3.0);
            if self.cursor.1 <= size + self.window.y {
                self.new_page();
            }
        }
    }

    /// Writes a line in the document.
    pub fn write_line(&mut self, words: &[&str], font: &Font, size: f64, spacing: f64) {
        let mut current_width = self.window.x;

        for word in words {
            let width = mm(current_width);
            let height = mm(self.cursor.1);

            self.layer.use_text(word.to_owned(), size as i64, width, height - mm(size), font.printpdf());
            current_width += font.text_width(word, size) + spacing;
        }

        self.new_line(size);
    }

    /// Goes to the beginning of the next line.
    pub fn new_line(&mut self, size: f64) {
        self.cursor.1 -= size;
    }

    /// Creates a new page and append it to the document.
    pub fn new_page(&mut self) {
        let page = self.document.add_page(mm(self.page_size.0), mm(self.page_size.1), "");
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

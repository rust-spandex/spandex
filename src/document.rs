//! This module allows to create beautiful documents.

use hyphenation::*;
use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use printpdf::{Mm, PdfDocument, PdfDocumentReference, PdfLayerReference, PdfPageReference, Pt};

use pulldown_cmark::{Event, Parser, Tag};

use crate::font::{Font, FontConfig};
use crate::typography::items::{Item, PositionedItem};
use crate::typography::paragraphs::{algorithm, itemize_paragraph, positionate_items};

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
            }

            3 => {
                self.subsubsections += 1;
                Some(self.subsubsections)
            }

            _ => {
                warn!("sub sub sub sections are not supported");
                None
            }
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
                }

                Event::Start(Tag::Item) => {
                    content.push_str(" - ");
                }

                Event::Text(ref text) => {
                    content.push(' ');
                    content.push_str(text);
                }

                Event::End(Tag::Paragraph) | Event::End(Tag::Item) => {
                    self.write_paragraph(&content, font_config.regular, current_size);
                    self.new_line(current_size);

                    content.clear();
                    current_size = size;
                }

                Event::End(Tag::Header(_)) => {
                    self.write_paragraph(&content, font_config.bold, current_size);
                    self.new_line(current_size);

                    content.clear();
                    current_size = size;
                }

                _ => (),
            }
            trace!("{:?}", event);
        }
    }

    /// Writes content on the document.
    pub fn write_content(&mut self, content: &str, font: &Font, size: f64) {
        for paragraph in content.split("\n") {
            self.layout_paragraph(paragraph, &font, size);
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
                self.write_line(
                    &words,
                    font,
                    size,
                    3.2 + remaining_width / ((words.len() - 1) as f64),
                );

                words.clear();
                words.push(remaining);

                if self.cursor.1 <= size + self.window.y {
                    self.new_page();
                }
            }
        }

        if !words.is_empty() {
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

            self.layer.use_text(
                word.to_owned(),
                size as i64,
                width,
                height - mm(size),
                font.printpdf(),
            );
            current_width += font.text_width(word, size) + spacing;
        }

        self.new_line(size);
    }

    pub fn layout_paragraph(&mut self, words: &str, font: &Font, font_size: f64) {
        fn get_line_length(total: i64) -> Vec<crate::units::Pt> {
            let mut lines: Vec<crate::units::Pt> = Vec::new();
            let base = 300.0;

            for x in -(total / 2)..(total / 2) {
                lines.push(crate::units::Pt(base + (x + 3).pow(2) as f64));
            }

            lines
        }

        if let Ok(en_us) = Standard::from_embedded(Language::EnglishUS) {
            let indentation = crate::units::Pt(18.0);

            let paragraph = itemize_paragraph(words, indentation, &font, font_size, &en_us);

            println!("Paragraph itemized into {:?} items.", paragraph.items.len());

            let lines_length = get_line_length(14); // vec![crate::units::Pt(400.0)];
            let breakpoints = algorithm(&paragraph, &lines_length);

            println!("{:?} breakpoints found.", breakpoints.len());

            let positions = positionate_items(&paragraph.items, &lines_length, &breakpoints);

            println!("Done positionating items. Laying out.");

            for line in positions {
                self.layout_line(&line, &font, font_size);
                self.new_line(font_size);
            }
        }
    }

    pub fn layout_line(&mut self, line_items: &Vec<PositionedItem>, font: &Font, font_size: f64) {
        let y = mm(self.cursor.1) - mm(font_size);

        for item in line_items {
            self.layer.use_text(
                item.glyph.to_string(),
                font_size as i64,
                mm(self.window.x + item.horizontal_offset.0),
                y,
                font.printpdf(),
            );
        }
    }

    /// Goes to the beginning of the next line.
    pub fn new_line(&mut self, size: f64) {
        self.cursor.1 -= size;
    }

    /// Creates a new page and append it to the document.
    pub fn new_page(&mut self) {
        let page = self
            .document
            .add_page(mm(self.page_size.0), mm(self.page_size.1), "");
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

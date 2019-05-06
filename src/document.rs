//! This module allows to create beautiful documents.

use std::fmt;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;

use printpdf::{PdfDocument, PdfDocumentReference, PdfPageReference, PdfLayerReference};

use pulldown_cmark::{Event, Tag, Parser};

use crate::typography::Word;
use crate::typography::justification::{Justifier, NaiveJustifier};
use crate::font::{Font, FontConfig, FontStyle};
use crate::units::{Pt, Sp};
use crate::parser::Ast;

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
        self.render_aux(ast, font_config, size, FontStyle::regular(), vec![]);
    }

    /// Renders an ast to the document with a certain buffer.
    fn render_aux(&mut self, ast: &Ast, font_config: &FontConfig, size: Sp, current_style: FontStyle, buffer: Vec<Word>) -> Vec<Word>{

        let mut buffer = buffer;

        match ast {
            Ast::Title { level, content } => {
                let size = size + Pt(3.0 * (4 - level) as f64).into();
                let buffer = self.render_aux(content, font_config, size, current_style.bold(), vec![]);
                self.write_paragraph::<NaiveJustifier>(&buffer, font_config, size);
                self.new_line(size);
            },

            Ast::Bold(content) => {
                buffer = self.render_aux(content, font_config, size, current_style.bold(), buffer);
            },

            Ast::Italic(content) => {
                buffer = self.render_aux(content, font_config, size, current_style.italic(), buffer);
            },

            Ast::InlineMath(_content) => {
                unimplemented!();
            },

            Ast::Text(content) => {
                buffer.push(Word { string: content.clone(), font_style: current_style });
            },

            Ast::Group(children) => {
                for child in children {
                    buffer = self.render_aux(child, font_config, size, current_style, buffer);
                }
            },

            Ast::Paragraph(children) => {
                for child in children {
                    buffer = self.render_aux(child, font_config, size, current_style, buffer);
                }
                self.write_paragraph::<NaiveJustifier>(&buffer, font_config, size);
                buffer = vec![];
                self.new_line(size);
            }

            Ast::Newline | Ast::Error(_) => (),
        }

        buffer
    }

    /// Writes markdown content on the document.
    pub fn write_markdown(&mut self, markdown: &str, font_config: &FontConfig, size: Sp) {

        let mut current_size = size;
        let mut content = vec![];
        let mut current_style = FontStyle::regular();

        let parser = Parser::new(markdown);

        for event in parser {
            match event {
                Event::Start(Tag::Header(i)) => {
                    current_style = current_style.bold();
                    if self.counters.increment(i).is_some() {
                        content.push(Word::new(format!("{}", self.counters), current_style));
                    }

                    current_size = size + Pt(3.0 * (4 - i) as f64).into();
                },

                Event::Start(Tag::Item) => {
                    content.push(Word::new(" - ".to_owned(), current_style));
                },

                Event::Start(Tag::Emphasis) => {
                    current_style = current_style.italic();
                },

                Event::End(Tag::Emphasis) => {
                    current_style = current_style.unitalic();
                }

                Event::Start(Tag::Strong) => {
                    current_style = current_style.bold();
                },

                Event::End(Tag::Strong) => {
                    current_style = current_style.unbold();
                }

                Event::Text(ref text) => {
                    content.push(Word::new(" ".to_owned(), current_style));
                    content.push(Word::new(text.to_string(), current_style));
                },

                Event::End(Tag::Paragraph) | Event::End(Tag::Item) => {
                    self.write_paragraph::<NaiveJustifier>(&content, font_config, current_size);
                    self.new_line(current_size);

                    content.clear();
                    current_size = size;
                },

                Event::End(Tag::Header(_)) => {
                    current_style = current_style.unbold();
                    self.write_paragraph::<NaiveJustifier>(&content, font_config, current_size);
                    self.new_line(current_size);

                    content.clear();
                    current_size = size;
                },

                _ => (),
            }
        }
    }

    /// Writes content on the document.
    pub fn write_content(&mut self, content: &str, font_config: &FontConfig, size: Sp) {
        for paragraph in content.split("\n") {
            let words = paragraph
                .split_whitespace()
                .map(|x| Word::new(x.to_string(), FontStyle::regular()))
                .collect::<Vec<_>>();

            self.write_paragraph::<NaiveJustifier>(&words, font_config, size);
            self.new_line(size);
        }
    }

    /// Writes a paragraph on the document.
    pub fn write_paragraph<J: Justifier>(&mut self, paragraph: &[Word], font_config: &FontConfig, size: Sp) {
        let size_i64 = Into::<Pt>::into(size).0 as i64;

        let justified = J::justify(paragraph, self.window.width, font_config, size);

        for line in justified {
            for glyph in line {

                let font = font_config.for_style(glyph.1);
                self.layer.use_text(
                    glyph.0.to_string(),
                    size_i64,
                    (self.window.x + glyph.2).into(),
                    self.cursor.1.into(),
                    font.printpdf());

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

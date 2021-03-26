//! This module allows to create beautiful documents.

pub mod configuration;
pub mod counters;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use printpdf::{PdfDocument, PdfDocumentReference, PdfLayerReference, PdfPageReference, Pt};
use spandex_hyphenation::load::Load;
use spandex_hyphenation::{Language, Standard};

use crate::document::counters::Counters;
use crate::fonts::configuration::FontConfig;
use crate::fonts::Font;
use crate::layout::paragraphs::justification::{Justifier, LatexJustifier, NaiveJustifier};
use crate::layout::paragraphs::utils::ast::itemize_ast;
use crate::parser::ast::Ast;

/// The window that is the part of the page on which we're allowed to write.
#[derive(Copy, Clone)]
pub struct Window {
    /// The x coordinate of the window, in pt.
    pub x: Pt,

    /// The y coordinate of the window, in pt.
    pub y: Pt,

    /// The width of the window, in pt.
    pub width: Pt,

    /// The height of the window, in pt.
    pub height: Pt,
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
    cursor: (Pt, Pt),

    /// The current page size, in pt.
    page_size: (Pt, Pt),

    /// The counters of the document
    counters: Counters,
}

impl Document {
    /// Creates a new pdf document from its name and its size in pt.
    pub fn new<T: Into<Pt>, U: Into<Pt>>(
        name: &str,
        width: T,
        height: U,
        window: Window,
    ) -> Document {
        let width: Pt = width.into();
        let height: Pt = height.into();

        let (document, page, layer) = PdfDocument::new(name, width.into(), height.into(), "");

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

    /// Renders an AST to the document.
    pub fn render(&mut self, ast: &Ast, font_config: &FontConfig, size: Pt) {
        let en = Standard::from_embedded(Language::EnglishUS).unwrap();

        match ast {
            Ast::File(_, children) => {
                for child in children {
                    self.render(child, font_config, size);
                }
            }

            Ast::Title { level, children } => {
                self.counters.increment(*level as usize);
                let mut new_children = vec![Ast::Text(format!("{}  ", self.counters))];
                new_children.extend_from_slice(children);
                let new_ast = Ast::Title {
                    level: *level,
                    children: new_children,
                };
                self.write_paragraph::<LatexJustifier>(&new_ast, font_config, size, &en);
                self.new_line(size);
            }

            Ast::Paragraph(_) => {
                self.write_paragraph::<LatexJustifier>(ast, font_config, size, &en);
                self.new_line(size);
                self.new_line(size);
            }

            Ast::UnorderedList(items) => {
                for item in items {
                    self.render(item, font_config, size);
                }
                self.new_line(size);
            }

            Ast::UnorderedListItem(_) => {
                // The LatexJustifier caused a problem with the example 'main.dex' file here, and only
                // showed 2 lines when there should have been 3. The NaiveJustifier doesn't have this
                // problem.
                // The problem text is as follows:
                //├─┬ UnorderedListItem
                //│ └── Text("Unordered list item\nstill part of list item (putting a dash at the start of a line with no space after it stops subsequent items from parsing for some reason, need to add a test for this and ﬁx)")
                self.write_paragraph::<NaiveJustifier>(ast, font_config, size, &en);
                self.new_line(size);
            }

            _ => (),
        }
    }

    /// Writes content on the document.
    pub fn write_content(&mut self, content: &str, font_config: &FontConfig, size: Pt) {
        let en = Standard::from_embedded(Language::EnglishUS).unwrap();

        for paragraph in content.split('\n') {
            let ast = Ast::Text(paragraph.to_owned());
            self.write_paragraph::<LatexJustifier>(&ast, font_config, size, &en);
            self.new_line(size);
        }
    }

    /// Writes a paragraph on the document.
    pub fn write_paragraph<J: Justifier>(
        &mut self,
        paragraph: &Ast,
        font_config: &FontConfig,
        size: Pt,
        dict: &Standard,
    ) {
        let paragraph = itemize_ast(paragraph, font_config, size, dict, Pt(0.0));
        let justified = J::justify(&paragraph, self.window.width);

        for line in justified {
            for glyph in line {
                self.layer.use_text(
                    glyph.0.glyph.to_string(),
                    Into::<Pt>::into(glyph.0.scale).0 as f64,
                    (self.window.x + glyph.1).into(),
                    self.cursor.1.into(),
                    glyph.0.font.printpdf(),
                );
            }

            self.new_line(size);
            self.cursor.0 = self.window.x;

            if self.cursor.1 <= size + self.window.y {
                self.new_page();
            }
        }
    }

    /// Writes a line in the document.
    pub fn write_line(&mut self, words: &[&str], font: &Font, size: Pt, spacing: Pt) {
        let size_f64 = Into::<Pt>::into(size).0 as f64;
        let mut current_width = self.window.x;

        for word in words {
            let width = current_width;
            let height = self.cursor.1;

            self.layer.use_text(
                word.to_owned(),
                size_f64,
                width.into(),
                (height - size).into(),
                font.printpdf(),
            );
            current_width += font.text_width(word, size) + spacing;
        }

        self.new_line(size);
    }

    /// Goes to the beginning of the next line.
    pub fn new_line(&mut self, size: Pt) {
        self.cursor.1 -= size;
    }

    /// Creates a new page and append it to the document.
    pub fn new_page(&mut self) {
        let page = self
            .document
            .add_page(self.page_size.0.into(), self.page_size.1.into(), "");
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

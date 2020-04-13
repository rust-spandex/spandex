//! This module allows to create beautiful documents.

pub mod configuration;
pub mod counters;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use hyphenation::load::Load;
use hyphenation::{Language, Standard};
use printpdf::{PdfDocument, PdfDocumentReference, PdfLayerReference, PdfPageReference, Pt};

use crate::document::counters::Counters;
use crate::fonts::configuration::FontConfig;
use crate::fonts::Font;
use crate::layout::paragraphs::justification::LatexJustifier;
use crate::layout::{write_paragraph, Layout};
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
    pub layer: PdfLayerReference,

    /// The window on which we're allowed to write on the page.
    pub window: Window,

    /// The cursor, the position where we supposed to write next.
    pub cursor: (Pt, Pt),

    /// The current page size, in pt.
    pub page_size: (Pt, Pt),

    /// The counters of the document
    counters: Counters,

    /// The layout of the document.
    pub layout: Box<dyn Layout>,
}

impl Document {
    /// Creates a new pdf document from its name and its size in pt.
    pub fn new<T: Into<Pt>, U: Into<Pt>>(
        name: &str,
        width: T,
        height: U,
        window: Window,
        layout: Box<dyn Layout>,
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
            layout,
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
            Ast::Group(children) => {
                for child in children {
                    self.render(child, font_config, size);
                }
            }

            Ast::Title { level, content } => {
                self.counters.increment(*level as usize);
                match &**content {
                    Ast::Group(children) => {
                        let mut new_children = vec![Ast::Text(format!("{}  ", self.counters))];
                        new_children.extend_from_slice(children);
                        let new_ast = Ast::Title {
                            level: *level,
                            content: Box::new(Ast::Group(new_children)),
                        };
                        write_paragraph::<LatexJustifier>(&new_ast, font_config, size, &en, self);
                    }
                    _ => write_paragraph::<LatexJustifier>(ast, font_config, size, &en, self),
                }
                self.new_line(size);
            }

            Ast::Paragraph(_) => {
                write_paragraph::<LatexJustifier>(ast, font_config, size, &en, self);
                self.new_line(size);
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
            write_paragraph::<LatexJustifier>(&ast, font_config, size, &en, self);
            self.new_line(size);
        }
    }

    /// Writes a line in the document.
    pub fn write_line(&mut self, words: &[&str], font: &Font, size: Pt, spacing: Pt) {
        let size_i64 = Into::<Pt>::into(size).0 as i64;
        let mut current_width = self.window.x;

        for word in words {
            let width = current_width;
            let height = self.cursor.1;

            self.layer.use_text(
                word.to_owned(),
                size_i64,
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

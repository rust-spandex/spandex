//! Logic for laying out the various pieces that make up a document.

pub mod constants;
pub mod errors;
pub mod pages;
pub mod paragraphs;

use printpdf::Pt;
use spandex_hyphenation::Standard;

use crate::document::Document;
use crate::fonts::configuration::FontConfig;
use crate::fonts::Font;
use crate::layout::pages::columns::Column;
use crate::layout::pages::Page;
use crate::layout::paragraphs::justification::Justifier;
use crate::layout::paragraphs::utils::ast::itemize_ast;
use crate::layout::paragraphs::Paragraph;
use crate::parser::ast::Ast;

/// A glyph with its font style.
#[derive(Debug, Clone)]
pub struct Glyph<'a> {
    /// The content of the word.
    pub glyph: char,

    /// The font style of the word.
    pub font: &'a Font,

    /// The size of the font.
    pub scale: Pt,
}

impl<'a> Glyph<'a> {
    /// Creates a new word from a string and a font style.
    pub fn new(glyph: char, font: &'a Font, scale: Pt) -> Glyph<'a> {
        Glyph { glyph, font, scale }
    }
}

/// Typesetting properties of a layout.
pub struct Properties {
    /// The distance between two paragraphs.
    pub paragraph_skip: Pt,

    /// The distance between two lines of a same paragraph.
    pub line_skip: Pt,

    /// The height of a line within a paragraph.
    pub line_height: Pt,
}

/// A layout holds all the pages and their columns that make up the document.
/// It is responsible for knowing where the virtual cursor is at all time,
/// which corresponds to where the next information is to be laid out.
/// A layout is defined by a set of typesetting properties and some logic
/// for generating the columns structure of new pages, e.g. depending on their
/// number. This allows to create different design for arbitrary page numbers.
pub trait Layout {
    /// Returns the current active column of the layout, which is where the
    /// virtual cursor sits at.
    fn current_column(&mut self) -> Column;

    /// Forces the layout to move the virtual cursor to the next available
    /// column.
    fn move_to_next_column(&mut self);

    /// Returns the next available column within the layout.
    fn get_next_column(&mut self) -> Column;

    /// Returns the current page of the layout, which is the page holding
    /// the current column.
    fn get_current_page(&mut self) -> Option<&Page>;

    /// Returns the page directly after the current page. A new page
    /// is created if the next page does not exist.
    fn get_next_page(&mut self) -> Option<&Page>;

    /// Appends a new page at the end of the document and sets the
    /// cursor to its first column.
    fn create_and_jump_to_last_page(&mut self) -> &Page;

    /// Returns a list of columns that make up a new page of the layout.
    fn get_columns_layout(&mut self) -> Vec<Column>;

    /// Lays out a paragraph into the current column.
    fn layout_paragraph(&self, paragraph: &Paragraph);

    /// Returns the properties of the layout.
    fn properties(&self) -> &Properties;

    /// Moves the cursor to the next available line for typesetting in
    /// the document. This also takes care of refreshing the current
    /// column and page.
    fn move_to_next_line(&mut self);

    /// Creates a new page at the end of the document without jumping
    /// to it.
    fn create_new_page_at_the_end(&mut self) -> &Page;

    /// Returns the index of the page on which the cursor currently is.
    fn current_page_index(&self) -> usize;

    /// Returns the i-th page in the document.
    fn get_ith_page(&mut self, i: usize) -> &Page;
}

/// A simple implementation of a paragraph where all pages are made up of
/// one single column.
pub struct TwoColumnLayout {
    pages: Vec<Page>,
    current_page_index: usize,
    current_column_index: usize,
    properties: Properties,
}

impl Layout for TwoColumnLayout {
    fn current_page_index(&self) -> usize {
        self.current_page_index
    }

    fn current_column(&mut self) -> Column {
        let current_page = &mut self.pages[self.current_page_index];

        current_page.get_current_column()
    }

    fn move_to_next_column(&mut self) {
        let _next_column = self.get_next_column();
        self.current_column_index += 1;
    }

    fn get_next_column(&mut self) -> Column {
        println!("Requesting next page...");
        let current_page = &mut self.pages[self.current_page_index];

        match current_page.get_next_column() {
            Some(next_column) => next_column,
            None => {
                // Jump to the fist column of the next page or create it if it doesn't exist.
                match self.get_next_page() {
                    Some(next_page) => next_page.get_first_column(),
                    None => {
                        let new_page = self.create_and_jump_to_last_page();
                        new_page.get_first_column()
                    }
                }
            }
        }
    }

    fn get_current_page(&mut self) -> Option<&Page> {
        if self.current_page_index >= self.pages.len() {
            None
        } else {
            Some(&self.pages[self.current_page_index])
        }
    }

    fn get_next_page(&mut self) -> Option<&Page> {
        self.current_page_index += 1;

        self.get_current_page()
    }

    fn get_ith_page(&mut self, i: usize) -> &Page {
        if i < self.pages.len() {
            &self.pages[i]
        } else {
            self.create_new_page_at_the_end()
        }
    }

    fn create_and_jump_to_last_page(&mut self) -> &Page {
        println!("Creating page {:?}", self.pages.len() as i32 + 1);
        let new_page = Page::new(self.get_columns_layout(), self.pages.len() as i32 + 1);

        self.pages.push(new_page);
        self.current_page_index = self.pages.len() - 1;

        &self.pages[self.current_page_index]
    }

    fn create_new_page_at_the_end(&mut self) -> &Page {
        println!("Creating page {:?}", self.pages.len() as i32 + 1);
        let new_page = Page::new(self.get_columns_layout(), self.pages.len() as i32 + 1);

        self.pages.push(new_page);

        &self.pages[self.current_page_index]
    }

    fn get_columns_layout(&mut self) -> Vec<Column> {
        vec![Column::new(Pt(250.0), Pt(50.0), Pt(400.0), Pt(200.0))]
    }

    fn layout_paragraph(&self, _paragraph: &Paragraph) {
        // Todo.
    }

    fn properties(&self) -> &Properties {
        &self.properties
    }

    fn move_to_next_line(&mut self) {
        unimplemented!()
    }
}

impl Default for TwoColumnLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Travels the layout until it finds the i-th column starting from a given
/// column.
fn get_next_ith_column_from_page(layout: &mut dyn Layout, page_index: usize, i: usize) -> Column {
    let current_page = layout.get_ith_page(page_index);

    match current_page.get_ith_column_from_current(i) {
        Some(target_column) => target_column,
        None => {
            // Find the missing columns on the next pages.
            let missing_columns_count = i - current_page.columns.len();

            get_next_ith_column_from_page(layout, page_index + 1, missing_columns_count)
        }
    }
}

/// Travels the layout until it finds the i-th column from where the cursor
/// currently is.
fn get_next_ith_column_from_current_page(layout: &mut dyn Layout, i: usize) -> Column {
    let current_page_index = layout.current_page_index();
    get_next_ith_column_from_page(layout, current_page_index, i)
}

/// Computes the width of a target line. If the line cannot fit in within the
/// current column, this function will explore the next columns until the line
/// can fit.
#[allow(dead_code)]
fn get_line_length_from_current_position(
    layout: &mut dyn Layout,
    column_index: usize,
    line_offset: usize,
) -> Pt {
    let Pt(line_height) = layout.properties().line_height;
    let Pt(line_skip) = layout.properties().line_skip;

    let line_offset = line_offset as f64;
    let current_column = get_next_ith_column_from_current_page(layout, column_index);
    let Pt(vertical_position) = current_column.current_vertical_position;

    let bouding_box_bottom_line =
        Pt(vertical_position + line_height * (line_offset + 1.0) + line_skip * line_offset);

    if bouding_box_bottom_line <= current_column.height {
        // The sought line can fit in this column.
        current_column.width
    } else {
        // The sought line cannot fit in this column.
        let Pt(fitting_lines) = current_column.height / (line_height + line_skip);
        let fitting_lines = fitting_lines.floor();

        let remaining_lines_to_fit = (line_offset - fitting_lines) as usize;

        get_line_length_from_current_position(layout, column_index + 1, remaining_lines_to_fit)
    }
}

/// Returns a column from the layout that can hold a line given by its offset.
pub fn get_column_for_line(layout: &mut dyn Layout, line_offset: usize) -> (usize, Column) {
    get_column_for_line_starting_at(layout, 0, line_offset)
}

fn get_column_for_line_starting_at(
    layout: &mut dyn Layout,
    column_index: usize,
    line_offset: usize,
) -> (usize, Column) {
    let Pt(line_height) = layout.properties().line_height;
    let Pt(line_skip) = layout.properties().line_skip;

    let line_offset = line_offset as f64;
    let current_column = get_next_ith_column_from_current_page(layout, column_index);
    let Pt(vertical_position) = current_column.current_vertical_position;

    let bouding_box_bottom_line =
        Pt(vertical_position + line_height * (line_offset + 1.0) + line_skip * line_offset);

    if bouding_box_bottom_line <= current_column.height {
        // The sought line can fit in this column.
        (column_index, current_column)
    } else {
        // The sought line cannot fit in this column.
        let Pt(fitting_lines) = current_column.height / (line_height + line_skip);
        let fitting_lines = fitting_lines.floor();

        let remaining_lines_to_fit = (line_offset - fitting_lines) as usize;

        get_column_for_line_starting_at(layout, column_index + 1, remaining_lines_to_fit)
    }
}

impl TwoColumnLayout {
    /// Creates a new two-column layout.
    pub fn new() -> TwoColumnLayout {
        TwoColumnLayout {
            pages: vec![Page::new(
                vec![
                    Column::new(Pt(0.0), Pt(50.0), Pt(200.0), Pt(200.0)),
                    Column::new(Pt(300.0), Pt(100.0), Pt(150.0), Pt(200.0)),
                ],
                1,
            )],
            current_page_index: 0,
            current_column_index: 0,
            properties: Properties {
                paragraph_skip: Pt(50.0),
                line_skip: Pt(2.0),
                line_height: Pt(10.0),
            },
        }
    }
}

/// Writes a paragraph on the document.
pub fn write_paragraph<J: Justifier>(
    paragraph: &Ast,
    font_config: &FontConfig,
    size: Pt,
    dict: &Standard,
    document: &mut Document,
) {
    let paragraph = itemize_ast(paragraph, font_config, size, dict, Pt(0.0));
    let justified = J::justify(&paragraph, &mut *document.layout);

    for line in justified {
        for glyph in line {
            document.layer.use_text(
                glyph.0.glyph.to_string(),
                Into::<Pt>::into(glyph.0.scale).0 as i64,
                (document.window.x + glyph.1).into(),
                document.cursor.1.into(),
                glyph.0.font.printpdf(),
            );
        }

        document.new_line(size);
        document.cursor.0 = document.window.x;

        if document.cursor.1 <= size + document.window.y {
            document.new_page();
        }
    }
}

/// Unit tests for layouts.
#[cfg(test)]
mod tests {
    use crate::layout::{get_line_length_from_current_position, Layout, TwoColumnLayout};
    use printpdf::Pt;

    use crate::Result;

    #[test]
    fn test_line_length_of_first_line() -> Result<()> {
        let mut layout: Layout = TwoColumnLayout::new();

        let line_length = get_line_length_from_current_position(&mut layout, 0, 0);

        assert_eq!(line_length, Pt(200.0));

        Ok(())
    }

    #[test]
    fn test_line_length_of_second_line() -> Result<()> {
        let mut layout: Layout = TwoColumnLayout::new();

        let line_length = get_line_length_from_current_position(&mut layout, 0, 1);

        assert_eq!(line_length, Pt(200.0));

        Ok(())
    }

    #[test]
    fn test_line_length_with_overflow_on_columns() -> Result<()> {
        let mut layout: dyn Layout = TwoColumnLayout::new();

        let line_length = get_line_length_from_current_position(&mut layout, 0, 19);

        assert_eq!(line_length, Pt(150.0));

        Ok(())
    }

    #[test]
    fn test_line_length_with_overflow_on_pages() -> Result<()> {
        let mut layout: dyn Layout = TwoColumnLayout::new();

        let line_length = get_line_length_from_current_position(&mut layout, 0, 34);

        assert_eq!(line_length, Pt(200.0));

        Ok(())
    }

    #[test]
    fn test_line_length_with_overflow_on_pages_with_offset() -> Result<()> {
        let mut layout: dyn Layout = TwoColumnLayout::new();

        let line_length = get_line_length_from_current_position(&mut layout, 0, 33);

        assert_eq!(line_length, Pt(200.0));

        Ok(())
    }
}

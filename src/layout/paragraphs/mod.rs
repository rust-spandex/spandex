pub mod engine;
pub mod graph;
pub mod items;
pub mod justification;
pub mod ligatures;
pub mod utils;

use std::slice::Iter;

use crate::layout::paragraphs::items::Item;

/// Holds a list of items describing a paragraph.
#[derive(Debug, Default)]
pub struct Paragraph<'a> {
    /// Sequence of items representing the structure of the paragraph.
    pub items: Vec<Item<'a>>,
}

impl<'a> Paragraph<'a> {
    /// Instantiates a new paragraph.
    pub fn new() -> Paragraph<'a> {
        Paragraph { items: Vec::new() }
    }

    /// Pushes an item at the end of the paragraph.
    pub fn push(&mut self, item: Item<'a>) {
        self.items.push(item)
    }

    /// Returns an iterator to the items of the paragraph.
    pub fn iter(&self) -> Iter<Item> {
        self.items.iter()
    }
}

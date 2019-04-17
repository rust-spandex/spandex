//! Mathematical logic for typesetting a sequence of words which have a
//! semantics of "paragraph". That is, the logic to split a sequence of
//! words into lines.
use crate::font::Font;
use crate::typography::items::Item;
use crate::units::{Sp, PLUS_INFINITY};
use hyphenation::*;
use std::vec::Vec;

/// Holds a list of items describing a paragraph.
pub struct Paragraph {
    /// Sequence of items representing the structure of the paragraph.
    pub items: Vec<Item>,
}

impl Paragraph {
    /// Instantiates a new paragraph.
    pub fn new() -> Paragraph {
        Paragraph { items: Vec::new() }
    }

    /// Pushes an item at the end of the paragraph.
    pub fn push(&mut self, item: Item) {
        println!("{:?}", item);
        self.items.push(item)
    }
}

/// Parses a string into a sequence of items.
pub fn itemize_paragraph(
    words: &str,
    indentation: Sp,
    font: &Font,
    font_size: f64,
    dictionary: &Standard,
) -> Paragraph {
    let mut paragraph = Paragraph::new();

    if indentation != Sp(0) {
        paragraph.push(Item::bounding_box(indentation, ' '));
    }

    let ideal_spacing = Sp(90_000);
    let hyphenation_width = Sp(80_000);
    let mut previous_glyph = 'c';
    let mut current_word = String::from("");

    // Turn each word of the paragraph into a sequence of boxes for
    // the caracters of the word. This includes potential punctuation
    // marks.
    for glyph in words.chars() {
        if glyph.is_whitespace() {
            paragraph.push(get_glue_from_context(previous_glyph, ideal_spacing));

            // Reached end of current word, handle hyphenation.
            let hyphenated = dictionary.hyphenate(&*current_word);
            let break_indices = &hyphenated.breaks;

            for (i, c) in current_word.chars().enumerate() {
                if break_indices.contains(&i) {
                    paragraph.push(Item::penalty(hyphenation_width, 20, true))
                }

                paragraph.push(Item::from_glyph(c, font, font_size));
            }

            current_word = String::from("");
        } else {
            current_word.push(glyph);
        }

        previous_glyph = glyph;
    }

    // Appends two items to ensure the end of any paragraph is
    // treated properly: a glue specifying the available space
    // at the right of the last tine, and a penalty item to
    // force a line break.
    paragraph.push(Item::glue(Sp(0), PLUS_INFINITY, Sp(0)));

    paragraph
}

/// Returns the glue based on the spatial context of the cursor.
fn get_glue_from_context(_previous_glyph: char, ideal_spacing: Sp) -> Item {
    // Todo: make this glue context dependent.
    Item::glue(ideal_spacing, Sp(0), Sp(0))
}

/// Unit tests for the paragraphs typesetting.
#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::typography::paragraphs::itemize_paragraph;
    use crate::units::Sp;
    use crate::{Error, Result};
    use hyphenation::*;
    use std::path::PathBuf;

    #[test]
    fn test_paragraph_itemization() -> Result<()> {
        let words = "Lorem ipsum dolor sit amet.";

        let en_us = Standard::from_embedded(Language::EnglishUS)?;

        let (_, font_manager) = Config::with_title("Test").init()?;

        let regular_font_name = "CMU Serif Roman";
        // let bold_font_name = "CMU Serif Bold";

        let font = font_manager
            .get(regular_font_name)
            .ok_or(Error::FontNotFound(PathBuf::from(regular_font_name)))?;

        // No indentation, meaning no leading empty box.
        let paragraph = itemize_paragraph(words, Sp(0), &font, 12.0, &en_us);
        assert_eq!(paragraph.items.len(), 27);

        // Indentated paragraph, implying the presence of a leading empty box.
        let paragraph = itemize_paragraph(words, Sp(120_000), &font, 12.0, &en_us);
        assert_eq!(paragraph.items.len(), 29);

        Ok(())
    }
}

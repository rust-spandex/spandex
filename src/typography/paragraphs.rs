//! Mathematical logic for typesetting a sequence of words which have a
//! semantics of "paragraph". That is, the logic to split a sequence of
//! words into lines.

use std::slice::Iter;

use hyphenation::*;

use crate::font::Font;
use crate::units::{Sp, PLUS_INFINITY};
use crate::typography::items::{
    Content,
    Item,
    INFINITELY_NEGATIVE_PENALTY,
    INFINITELY_POSITIVE_PENALTY,
};

const DASH_GLYPH: char = '-';

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
        self.items.push(item)
    }

    /// Returns an iterator to the items of the paragraph.
    pub fn iter(&self) -> Iter<Item> {
        self.items.iter()
    }
}

/// Parses a string into a sequence of items.
pub fn itemize_paragraph(
    words: &str,
    indentation: Sp,
    font: &Font,
    font_size: Sp,
    dictionary: &Standard,
) -> Paragraph {
    let mut paragraph = Paragraph::new();

    // Add trailing space to ensure the last word is treated.
    let words = format!("{}{}", words, " ");

    if indentation != Sp(0) {
        paragraph.push(Item::bounding_box(indentation, ' '));
    }

    let ideal_spacing = Sp(90_000);
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
                    paragraph.push(Item::penalty(Sp(0), 50, true))
                }

                paragraph.push(Item::from_glyph(c, font, font_size));

                if c == DASH_GLYPH {
                    paragraph.push(Item::penalty(Sp(0), 50, true))
                }
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
    paragraph.push(Item::penalty(Sp(0), INFINITELY_NEGATIVE_PENALTY, false));

    paragraph
}

/// Returns the glue based on the spatial context of the cursor.
fn get_glue_from_context(_previous_glyph: char, ideal_spacing: Sp) -> Item {
    // Todo: make this glue context dependent.
    Item::glue(ideal_spacing, Sp(0), Sp(0))
}

/// Finds all the legal breakpoints within a paragraph. A legal breakpoint
/// is an item index such that this item is either a peanalty which isn't
/// infinite or a glue following a bounding box.
pub fn find_legal_breakpoints(paragraph: &Paragraph) -> Vec<usize> {
    let mut legal_breakpoints: Vec<usize> = Vec::new();
    legal_breakpoints.push(0);

    let mut last_item_was_box = false;

    for (i, item) in paragraph.items.iter().enumerate() {
        match item.content {
            Content::Penalty { value, .. } => {
                if value < INFINITELY_POSITIVE_PENALTY {
                    legal_breakpoints.push(i);
                }

                last_item_was_box = false;
            }
            Content::Glue { .. } => {
                if last_item_was_box {
                    legal_breakpoints.push(i)
                }

                last_item_was_box = false;
            }
            Content::BoundingBox { .. } => last_item_was_box = true,
        }
    }

    legal_breakpoints
}

/// Computes the adjusment ratio of a line of items, based on their combined
/// width, stretchability and shrinkability. This essentially tells how much
/// effort has to be produce to fit the line to the desired width.
#[allow(dead_code)]
fn compute_adjustment_ratio(
    actual_length: Sp,
    desired_length: Sp,
    total_stretchability: Sp,
    total_shrinkability: Sp,
) -> f64 {
    if actual_length == desired_length {
        0.0
    } else if actual_length < desired_length {
        (desired_length.0 as f64 - actual_length.0 as f64) / total_stretchability.0 as f64
    } else {
        (desired_length.0 as f64 - actual_length.0 as f64) / total_shrinkability.0 as f64
    }
}

/// Unit tests for the paragraphs typesetting.
#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::typography::paragraphs::{find_legal_breakpoints, itemize_paragraph};
    use crate::units::{Pt, Sp};
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
        let paragraph = itemize_paragraph(words, Sp(0), &font, Pt(12.0).into(), &en_us);
        assert_eq!(paragraph.items.len(), 32);

        // Indentated paragraph, implying the presence of a leading empty box.
        let paragraph = itemize_paragraph(words, Sp(120_000), &font, Pt(12.0).into(), &en_us);
        assert_eq!(paragraph.items.len(), 33);

        Ok(())
    }

    #[test]
    fn test_legal_breakpoints() -> Result<()> {
        let words = "Lorem ipsum dolor sit amet.";

        let en_us = Standard::from_embedded(Language::EnglishUS)?;

        let (_, font_manager) = Config::with_title("Test").init()?;

        let regular_font_name = "CMU Serif Roman";
        // let bold_font_name = "CMU Serif Bold";

        let font = font_manager
            .get(regular_font_name)
            .ok_or(Error::FontNotFound(PathBuf::from(regular_font_name)))?;

        // Indentated paragraph, implying the presence of a leading empty box.
        let paragraph = itemize_paragraph(words, Sp(120_000), &font, Pt(12.0).into(), &en_us);

        let legal_breakpoints = find_legal_breakpoints(&paragraph);
        // [ ] Lorem ip-sum do-lor sit amet.
        assert_eq!(legal_breakpoints, [0, 1, 7, 10, 14, 17, 21, 25, 31, 32]);

        Ok(())
    }

    // #[test]
    // fn test_adjustment_ratio_computation() -> Result<()> {
    //     let words = "Lorem ipsum dolor sit amet.";

    //     let en_us = Standard::from_embedded(Language::EnglishUS)?;

    //     let (_, font_manager) = Config::with_title("Test").init()?;

    //     let regular_font_name = "CMU Serif Roman";
    //     // let bold_font_name = "CMU Serif Bold";

    //     let font = font_manager
    //         .get(regular_font_name)
    //         .ok_or(Error::FontNotFound(PathBuf::from(regular_font_name)))?;

    //     // Indentated paragraph, implying the presence of a leading empty box.
    //     let paragraph = itemize_paragraph(words, Sp(120_000), &font, 12.0, &en_us);
    //     // assert_eq!(paragraph.items.len(), 26);

    //     // TODO: compute the ratio by hand.

    //     Ok(())
    // }
}

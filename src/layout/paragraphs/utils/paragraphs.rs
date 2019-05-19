use crate::layout::constants::{DASH_GLYPH, DEFAULT_LINE_LENGTH, SPACE_WIDTH};
use crate::layout::paragraphs::items::{Content, Item};
use crate::layout::paragraphs::Paragraph;
use crate::layout::Glyph;
use hyphenation::*;
use printpdf::Pt;

/// Adds a word to a buffer.
pub fn add_word_to_paragraph<'a>(
    word: Vec<Glyph<'a>>,
    dictionary: &Standard,
    buffer: &mut Paragraph<'a>,
) {
    // Reached end of current word, handle hyphenation.
    let to_hyphenate = word
        .iter()
        .map(|x: &Glyph| x.glyph.to_string())
        .collect::<Vec<_>>()
        .join("");

    let hyphenated = dictionary.hyphenate(&to_hyphenate);
    let break_indices = &hyphenated.breaks;

    for (i, g) in word.iter().enumerate() {
        if break_indices.contains(&i) {
            buffer.push(Item::penalty(Pt(0.0), 50.0, true));
        }

        buffer.push(Item::from_glyph(g.clone()));

        if g.glyph == DASH_GLYPH {
            buffer.push(Item::penalty(Pt(0.0), 50.0, true));
        }
    }
}

/// Returns the glue based on the spatial context of the cursor.
pub fn glue_from_context(_previous_glyph: Option<Glyph>, ideal_spacing: Pt) -> Item {
    // Todo: make this glue context dependent.
    Item::glue(ideal_spacing, SPACE_WIDTH, SPACE_WIDTH * 0.5)
}

/// Returns the length of the line of given index, from a list of
/// potential line lengths. If the list is too short, the line
/// length will default to `DEFAULT_LINE_LENGTH`.
pub fn get_line_length(lines_length: &[Pt], index: usize) -> Pt {
    if index < lines_length.len() {
        lines_length[index]
    } else {
        *lines_length.first().unwrap_or(&DEFAULT_LINE_LENGTH)
    }
}

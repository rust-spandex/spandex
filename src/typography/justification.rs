//! This module contains the trait and implementation of justification algorithms.

use printpdf::Pt;

use crate::typography::items::Content;
use crate::typography::paragraphs::{algorithm, positionate_items, Paragraph, IDEAL_SPACING};
use crate::typography::Glyph;

/// An algorithm that justifies a paragraph.
pub trait Justifier {
    /// Justifies the paragraph passed as parameter.
    fn justify<'a>(paragraph: &'a Paragraph<'a>, text_width: Pt) -> Vec<Vec<(Glyph<'a>, Pt)>>;
}

/// A naive justifier, that goes to the next line once a word overtakes the text width.
pub struct NaiveJustifier;

impl Justifier for NaiveJustifier {
    fn justify<'a>(paragraph: &'a Paragraph<'a>, text_width: Pt) -> Vec<Vec<(Glyph<'a>, Pt)>> {
        let mut ret = vec![];
        let mut current_line = vec![];
        let mut current_word = vec![];
        let mut current_x = Pt(0.0);

        for item in paragraph.iter() {
            match item.content {
                Content::BoundingBox { .. } => {
                    current_x += item.width;
                    current_word.push(item);
                }
                Content::Glue { .. } => {
                    current_line.push(current_word);
                    current_x += item.width;
                    current_word = vec![];
                }
                Content::Penalty { .. } => (),
            }

            if current_x > text_width && current_line.len() > 1 {
                current_x = Pt(0.0);

                let last_word = current_line.pop().unwrap();

                let mut occupied_width = Pt(0.0);
                for word in &current_line {
                    for glyph in word {
                        occupied_width += glyph.width;
                    }
                }

                let available_space = text_width - occupied_width;

                let word_space = if current_line.len() > 1 {
                    available_space / (current_line.len() - 1) as f64
                } else {
                    IDEAL_SPACING
                };

                let mut current_x = Pt(0.0);
                let mut final_line = vec![];

                for word in current_line {
                    for item in &word {
                        if let Content::BoundingBox(ref glyph) = item.content {
                            final_line.push((glyph.clone(), current_x));
                            current_x += item.width;
                        }
                    }

                    // Put a space after the word
                    current_x += word_space;
                }

                ret.push(final_line);

                current_line = vec![last_word];
            }
        }

        let mut current_x = Pt(0.0);
        let mut final_line = vec![];

        // There is still content in current_line
        for word in current_line {
            for item in word {
                if let Content::BoundingBox(ref glyph) = item.content {
                    final_line.push((glyph.clone(), current_x));
                    current_x += item.width;
                }
            }
            current_x += IDEAL_SPACING;
        }

        ret.push(final_line);

        ret
    }
}

/// The LaTeX style justifier.
pub struct LatexJustifier;

impl Justifier for LatexJustifier {
    fn justify<'a>(paragraph: &Paragraph<'a>, text_width: Pt) -> Vec<Vec<(Glyph<'a>, Pt)>> {
        let lines_length = vec![text_width];
        let breakpoints = algorithm(&paragraph, &lines_length);
        let positioned_items = positionate_items(&paragraph.items, &lines_length, &breakpoints);

        let mut output = vec![];

        for items in positioned_items {
            let mut line = vec![];
            for item in items {
                line.push((item.glyph.clone(), item.horizontal_offset));
            }
            output.push(line);
        }

        output
    }
}

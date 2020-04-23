//! This module contains the trait and implementation of justification algorithms.

use printpdf::Pt;

use crate::layout::constants::IDEAL_SPACING;
use crate::layout::paragraphs::engine::{algorithm, positionate_items};
use crate::layout::paragraphs::items::Content;
use crate::layout::paragraphs::Paragraph;
use crate::layout::Glyph;
use crate::layout::Layout;

/// An algorithm that justifies a paragraph.
pub trait Justifier {
    /// Justifies the paragraph passed as parameter.
    fn justify<'a>(
        paragraph: &'a Paragraph<'a>,
        layout: &mut dyn Layout,
    ) -> Vec<Vec<(Glyph<'a>, Pt)>>;
}

/// A naive justifier, that goes to the next line once a word overtakes the text width.
pub struct NaiveJustifier;

impl Justifier for NaiveJustifier {
    fn justify<'a>(
        paragraph: &'a Paragraph<'a>,
        layout: &mut dyn Layout,
    ) -> Vec<Vec<(Glyph<'a>, Pt)>> {
        println!("Using naive justifier.");

        let mut ret = vec![];
        let mut current_line = vec![];
        let mut current_word = vec![];
        let mut current_x = Pt(0.0);
        let text_width = layout.current_column().width;

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
    fn justify<'a>(
        paragraph: &Paragraph<'a>,
        layout: &mut dyn Layout,
    ) -> Vec<Vec<(Glyph<'a>, Pt)>> {
        println!("Using LaTeX justifier.");
        println!("Layout: {:?}", layout.current_column().width);

        let breakpoints = algorithm(&paragraph, layout);
        let positioned_items = positionate_items(&paragraph.items, layout, &breakpoints);

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

/// The LaTeX style justifier.
pub struct SpandexJustifier;

impl Justifier for SpandexJustifier {
    fn justify<'a>(
        paragraph: &Paragraph<'a>,
        layout: &mut dyn Layout,
    ) -> Vec<Vec<(Glyph<'a>, Pt)>> {
        println!("Using SpanDeX justifier.");

        let breakpoints = algorithm(&paragraph, layout);
        let positioned_items = positionate_items(&paragraph.items, layout, &breakpoints);

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

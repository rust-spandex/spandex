//! This module contains the trait and implementation of justification algorithms.

use hyphenation::load::Load;
use hyphenation::{Standard, Language};

use crate::units::Sp;
use crate::font::{FontStyle, FontConfig};
use crate::typography::Word;
use crate::typography::items::Content;
use crate::typography::paragraphs::{Paragraph, itemize_paragraph};

/// An algorithm that justifies a paragraph.
pub trait Justifier {
    /// Computes the paragraph decomposition of the string and justifies it.
    fn justify(content: &[Word], text_width: Sp, font_config: &FontConfig, size: Sp) -> Vec<Vec<(char, FontStyle, Sp)>> {
        let en = Standard::from_embedded(Language::EnglishUS).unwrap();
        let paragraph = itemize_paragraph(&content, Sp(0), font_config, size, &en);
        Self::justify_paragraph(&paragraph, text_width)
    }

    /// Justifies the paragraph passed as parameter.
    fn justify_paragraph(paragraph: &Paragraph, text_width: Sp) -> Vec<Vec<(char, FontStyle, Sp)>>;
}

/// A naive justifier, that goes to the next line once a word overtakes the text width.
pub struct NaiveJustifier;

impl Justifier for NaiveJustifier {
    fn justify_paragraph(paragraph: &Paragraph, text_width: Sp) -> Vec<Vec<(char, FontStyle, Sp)>> {
        let mut ret = vec![];
        let mut current_line = vec![];
        let mut current_word = vec![];
        let mut current_x = Sp(0);

        for item in paragraph.iter().skip(1) {
            match item.content {
                Content::BoundingBox { .. } => {
                    current_x += item.width;
                    current_word.push(item);
                },
                Content::Glue { .. } => {
                    current_line.push(current_word);
                    current_x += Sp(200_000);
                    current_word = vec![];
                },
                Content::Penalty { .. } => (),
            }

            if current_x > text_width {

                current_x = Sp(0);

                debug_assert!(current_line.len() > 1);

                let last_word = current_line.pop().unwrap();

                let mut occupied_width = Sp(0);
                for word in &current_line {
                    for glyph in word {
                        occupied_width += glyph.width;
                    }
                }

                let available_space = text_width - occupied_width;
                let word_space = available_space / Sp((current_line.len() - 1) as i64);
                let mut current_x = Sp(0);
                let mut final_line = vec![];

                for word in current_line {
                    for item in &word {
                        match item.content {
                            Content::BoundingBox { glyph, font_style } => {
                                final_line.push((glyph, font_style, current_x));
                                current_x += item.width;
                            },

                            _ => (),
                        }
                    }

                    // Put a space after the word
                    current_x += word_space;
                }

                ret.push(final_line);

                current_line = vec![last_word];
            }
        }

        let mut current_x = Sp(0);
        let mut final_line = vec![];

        // There is still content in current_line
        for word in current_line {
            for item in word {
                match item.content {
                    Content::BoundingBox { glyph, font_style } => {
                        final_line.push((glyph, font_style, current_x));
                        current_x += item.width;
                    },
                    _ => (),
                }
            }
            current_x += Sp(200_000);
        }

        ret.push(final_line);

        ret
    }
}

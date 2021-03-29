//! Utility functions for manipulating an abstract syntax tree representing
//! a paragraph.

use crate::fonts::configuration::FontConfig;
use crate::fonts::styles::FontStyle;
use crate::layout::constants::{IDEAL_SPACING, PLUS_INFINITY};
use crate::layout::paragraphs::items::Item;
use crate::layout::paragraphs::utils::paragraphs::{add_word_to_paragraph, glue_from_context};
use crate::layout::paragraphs::Paragraph;
use crate::layout::Glyph;
use crate::parser::ast::Ast;
use printpdf::Pt;
use spandex_hyphenation::*;
use std::f64;

/// Parses an AST into a sequence of items.
pub fn itemize_ast<'a>(
    ast: &Ast,
    font_config: &'a FontConfig,
    size: Pt,
    dictionary: &Standard,
    indent: Pt,
) -> Paragraph<'a> {
    let mut p = Paragraph::new();
    let current_style = FontStyle::regular();

    if indent > Pt(0.0) {
        p.push(Item::glue(indent, Pt(0.0), Pt(0.0)));
    }

    itemize_ast_aux(ast, font_config, size, dictionary, current_style, &mut p);
    p
}

/// Parses an AST into a sequence of items.
pub fn itemize_ast_aux<'a>(
    ast: &Ast,
    font_config: &'a FontConfig,
    size: Pt,
    dictionary: &Standard,
    current_style: FontStyle,
    buffer: &mut Paragraph<'a>,
) {
    match ast {
        Ast::Title { level, children } => {
            let size = size + Pt(3.0 * ((4 - *level as isize).max(1)) as f64);
            for child in children {
                itemize_ast_aux(
                    child,
                    font_config,
                    size,
                    dictionary,
                    current_style.bold(),
                    buffer,
                );
            }

            end_line(buffer);
        }

        Ast::Bold(children) => {
            for child in children {
                itemize_ast_aux(
                    child,
                    font_config,
                    size,
                    dictionary,
                    current_style.bold(),
                    buffer,
                );
            }
        }

        Ast::Italic(children) => {
            for child in children {
                itemize_ast_aux(
                    child,
                    font_config,
                    size,
                    dictionary,
                    current_style.italic(),
                    buffer,
                );
            }
        }

        Ast::Text(content) => {
            let font = font_config.for_style(current_style);
            let ideal_spacing = IDEAL_SPACING;
            let mut previous_glyph = None;
            let mut current_word = vec![];

            // Turn each word of the paragraph into a sequence of boxes for the caracters of the
            // word. This includes potential punctuation marks.
            for c in content.chars() {
                if c.is_whitespace() {
                    add_word_to_paragraph(current_word, dictionary, buffer);
                    buffer.push(glue_from_context(previous_glyph, ideal_spacing));
                    current_word = vec![];
                } else {
                    current_word.push(Glyph::new(c, font, size));
                }

                previous_glyph = Some(Glyph::new(c, font, size));
            }

            // Current word is empty if content ends with a whitespace.

            if !current_word.is_empty() {
                add_word_to_paragraph(current_word, dictionary, buffer);
            }
        }

        Ast::File(_, children) => {
            for child in children {
                itemize_ast_aux(child, font_config, size, dictionary, current_style, buffer);
            }
        }

        Ast::Paragraph(children) => {
            for child in children {
                itemize_ast_aux(child, font_config, size, dictionary, current_style, buffer);
            }

            end_line(buffer);
        }

        Ast::UnorderedList(_) => {
            // This is handled in mod.rs, Document.render
        }

        Ast::UnorderedListItem { level, children } => {
            let bullet = " ".repeat(*level as usize) + "• ";

            itemize_ast_aux(
                &Ast::Text(bullet),
                font_config,
                size,
                dictionary,
                current_style,
                buffer,
            );

            for child in children {
                itemize_ast_aux(child, font_config, size, dictionary, current_style, buffer);
            }

            end_line(buffer);
        }

        _ => (),
    }
}

fn end_line(buffer: &mut Paragraph) {
    // Appends two items to ensure the end of any paragraph is treated properly: a glue
    // specifying the available space at the right of the last tine, and a penalty item to
    // force a line break.
    buffer.push(Item::glue(Pt(0.0), PLUS_INFINITY, Pt(0.0)));
    buffer.push(Item::penalty(Pt(0.0), f64::NEG_INFINITY, false));
}

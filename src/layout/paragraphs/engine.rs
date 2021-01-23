//! Mathematical logic for typesetting a sequence of words which have a
//! semantics of "paragraph". That is, the logic to split a sequence of
//! words into lines.

use std::collections::hash_map::{Entry, HashMap};
use std::f64;

use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::Dfs;
use petgraph::visit::IntoNodeIdentifiers;
use printpdf::Pt;

use crate::layout::paragraphs::items::{Content, Item, PositionedItem};
use crate::layout::paragraphs::Paragraph;
use crate::layout::Glyph;

use crate::layout::constants::{MAX_ADJUSTMENT_RATIO, MIN_ADJUSTMENT_RATIO};
use crate::layout::paragraphs::graph::Node;
use crate::layout::paragraphs::utils::linebreak::{
    compute_adjustment_ratio, compute_adjustment_ratios_with_breakpoints,
    create_node_for_feasible_breakpoint, is_forced_break, Measures,
};
use crate::layout::paragraphs::utils::paragraphs::get_line_length;

/// Finds the optimal sequence of breakpoints that minimize
/// the amount of demerits while breaking a paragraph down
/// into lines.
///
/// It returns the indexes of items which have been chosen as
/// breakpoints.
pub fn algorithm<'a>(paragraph: &'a Paragraph<'a>, lines_length: &[Pt]) -> Vec<usize> {
    let mut graph = StableGraph::<_, f64>::new();
    let mut sum_width = Pt(0.0);
    let mut sum_stretch = Pt(0.0);
    let mut sum_shrink = Pt(0.0);
    let mut best_adjustment_ratio_above_threshold = f64::MAX;
    let current_maximum_adjustment_ratio = f64::MAX;

    let mut lines_best_node = HashMap::new();
    let mut farthest_line: usize = 0;

    // Add an initial active node for the beginning of the paragraph.
    graph.add_node(Node {
        index: 0,
        line: 0,
        fitness: 1,
        total_width: Pt(0.0),
        total_stretch: Pt(0.0),
        total_shrink: Pt(0.0),
        total_demerits: 0.0,
    });

    for (b, item) in paragraph.items.iter().enumerate() {
        if item.width < Pt(0.0) {
            panic!("Item #{} has negative width.", b);
        }

        let mut can_break = false;

        match item.content {
            Content::BoundingBox { .. } => {
                sum_width += item.width;
            }
            Content::Glue {
                stretchability,
                shrinkability,
            } => {
                // We can only break at a glue if it is preceeded by
                // a bounding box.
                can_break =
                    b > 0 && matches!(paragraph.items[b-1].content, Content::BoundingBox { .. });

                if !can_break {
                    sum_width += item.width;
                    sum_shrink += shrinkability;
                    sum_stretch += stretchability;
                }
            }
            Content::Penalty { value, .. } => {
                can_break = value < f64::INFINITY;
            }
        }

        if !can_break {
            continue;
        }

        // Update the set of active nodes.

        let mut feasible_breakpoints: Vec<(Node, NodeIndex)> = Vec::new();
        let mut node_to_remove: Vec<NodeIndex> = Vec::new();

        for node in graph.node_identifiers() {
            if let Some(a) = graph.node_weight(node) {
                let line_shrink = sum_shrink - a.total_shrink;
                let line_stretch = sum_stretch - a.total_stretch;
                let actual_width = sum_width - a.total_width;

                let adjustment_ratio = compute_adjustment_ratio(
                    actual_width,
                    get_line_length(&lines_length, a.line),
                    line_stretch,
                    line_shrink,
                );

                if adjustment_ratio > current_maximum_adjustment_ratio {
                    best_adjustment_ratio_above_threshold =
                        adjustment_ratio.min(best_adjustment_ratio_above_threshold)
                }

                if adjustment_ratio < MIN_ADJUSTMENT_RATIO || is_forced_break(item) {
                    // Items from a to b cannot fit on the same line.
                    node_to_remove.push(node);
                }

                if (MIN_ADJUSTMENT_RATIO..=MAX_ADJUSTMENT_RATIO).contains(&adjustment_ratio) {
                    let measures_sum = Measures {
                        width: sum_width,
                        shrinkability: sum_shrink,
                        stretchability: sum_stretch,
                    };

                    let new_node = create_node_for_feasible_breakpoint(
                        b,
                        a,
                        adjustment_ratio,
                        &item,
                        &paragraph.items,
                        &measures_sum,
                    );

                    feasible_breakpoints.push((new_node, node));
                }
            }
        }

        // If there is a feasible break at b, then append the best such break
        // as an active node.
        if !feasible_breakpoints.is_empty() {
            let (mut last_best_node, mut last_node_parent_id) = feasible_breakpoints[0];

            for (node, parent_id) in feasible_breakpoints.iter() {
                if node.total_demerits < last_best_node.total_demerits {
                    last_best_node = *node;
                    last_node_parent_id = *parent_id;
                }
            }

            let inserted_node = graph.add_node(last_best_node);

            // Create a precedence relationship between a and the best node.
            graph.add_edge(
                inserted_node,
                last_node_parent_id,
                last_best_node.total_demerits,
            );

            match lines_best_node.entry(last_best_node.line) {
                Entry::Vacant(entry) => {
                    entry.insert((last_best_node, inserted_node));
                    farthest_line += 1;
                }

                Entry::Occupied(entry) => {
                    let (best_node_on_current_line, _) = entry.get();
                    if last_best_node.total_demerits < best_node_on_current_line.total_demerits {
                        lines_best_node
                            .insert(last_best_node.line, (last_best_node, inserted_node));
                    }
                }
            }
        }

        if let Content::Glue {
            shrinkability,
            stretchability,
        } = item.content
        {
            sum_width += item.width;
            sum_shrink += shrinkability;
            sum_stretch += stretchability;
        }
    }

    // TODO: handle situation where there's no option to fall within the window of
    // accepted adjustment ratios.

    // Follow the edges backwards.
    let mut result: Vec<usize> = Vec::new();

    if let Some((_, best_node_on_last_line)) = lines_best_node.get(&farthest_line) {
        let mut dfs = Dfs::new(&graph, *best_node_on_last_line);
        while let Some(node_index) = dfs.next(&graph) {
            // use a detached neighbors walker
            if let Some(node) = graph.node_weight(node_index) {
                result.push(node.index);
            }
        }
    }
    result.reverse();
    result
}

/// Generates a list of positioned items from a list of items making up a paragraph.
/// The generated list is ready to be rendered.
pub fn positionate_items<'a>(
    items: &[Item<'a>],
    line_lengths: &[Pt],
    breakpoints: &[usize],
) -> Vec<Vec<PositionedItem<'a>>> {
    let adjustment_ratios =
        compute_adjustment_ratios_with_breakpoints(items, line_lengths, breakpoints);
    let mut lines_breakdown: Vec<Vec<PositionedItem>> = Vec::new();

    for breakpoint_line in 0..(breakpoints.len() - 1) {
        let mut positioned_items: Vec<PositionedItem> = Vec::new();

        let breakpoint_index = breakpoints[breakpoint_line];
        let adjustment_ratio = adjustment_ratios[breakpoint_line].max(MIN_ADJUSTMENT_RATIO);
        let mut horizontal_offset = Pt(0.0);
        let beginning = if breakpoint_line == 0 {
            breakpoint_index
        } else {
            breakpoint_index + 1
        };

        let mut previous_glyph = None;

        let range = items
            .iter()
            .enumerate()
            .take(breakpoints[breakpoint_line + 1])
            .skip(beginning);

        for (p, item) in range {
            match items[p].content {
                Content::BoundingBox(ref glyph) => {
                    previous_glyph = Some(glyph.clone());
                    positioned_items.push(PositionedItem {
                        index: p,
                        line: breakpoint_line,
                        horizontal_offset,
                        width: item.width,
                        glyph: glyph.clone(),
                    });
                    horizontal_offset += item.width;
                }
                Content::Glue {
                    shrinkability,
                    stretchability,
                } => {
                    if p != beginning && p != breakpoints[breakpoint_line + 1] {
                        let width = item.width;

                        let gap = if adjustment_ratio < 0.0 {
                            width + shrinkability * adjustment_ratio
                        } else {
                            width + stretchability * adjustment_ratio
                        };

                        // TODO: add an option to handle the inclusion of glue.

                        horizontal_offset += gap;
                    }
                }
                Content::Penalty { .. } => {
                    if p == breakpoints[breakpoint_line + 1] && items[p].width > Pt(0.0) {
                        let glyph = previous_glyph.clone().unwrap();
                        positioned_items.push(PositionedItem {
                            index: p,
                            line: breakpoint_line,
                            horizontal_offset,
                            width: item.width,
                            glyph: Glyph {
                                glyph: '-',
                                font: glyph.font,
                                scale: glyph.scale,
                            },
                        })
                    }
                }
            }
        }

        lines_breakdown.push(positioned_items);
    }

    lines_breakdown
}

/// Unit tests for the paragraphs typesetting.
#[cfg(test)]
mod tests {

    use printpdf::Pt;
    use spandex_hyphenation::*;

    use crate::document::configuration::Config;
    use crate::layout::paragraphs::engine::algorithm;
    use crate::layout::paragraphs::items::Content;
    use crate::layout::paragraphs::utils::ast::itemize_ast;
    use crate::layout::paragraphs::utils::linebreak::{
        compute_adjustment_ratios_with_breakpoints, find_legal_breakpoints,
    };
    use crate::parser::ast::Ast;
    use crate::Result;

    #[test]
    fn test_paragraph_itemization() -> Result<()> {
        let words = "Lorem ipsum dolor sit amet.";
        let ast = Ast::Paragraph(vec![Ast::Text(words.into())]);

        let en_us = Standard::from_embedded(Language::EnglishUS)?;

        let (_, font_manager) = Config::with_title("Test").init()?;
        let config = font_manager.default_config();

        // No indentation, meaning no leading empty box.
        let paragraph = itemize_ast(&ast, &config, Pt(10.0), &en_us, Pt(0.0));
        assert_eq!(paragraph.items.len(), 31);

        // Indentated paragraph, implying the presence of a leading empty box.
        let paragraph = itemize_ast(&ast, &config, Pt(10.0), &en_us, Pt(7.5));
        assert_eq!(paragraph.items.len(), 32);

        Ok(())
    }

    #[test]
    fn test_legal_breakpoints() -> Result<()> {
        let words = "Lorem ipsum dolor sit amet.";
        let ast = Ast::Paragraph(vec![Ast::Text(words.into())]);

        let en_us = Standard::from_embedded(Language::EnglishUS)?;

        let (_, font_manager) = Config::with_title("Test").init()?;
        let config = font_manager.default_config();

        // Indentated paragraph, implying the presence of a leading empty box.
        let paragraph = itemize_ast(&ast, &config, Pt(10.0), &en_us, Pt(7.5));

        let legal_breakpoints = find_legal_breakpoints(&paragraph);
        // [ ] Lorem ip-sum do-lor sit amet.
        assert_eq!(legal_breakpoints, [0, 6, 9, 13, 16, 20, 24, 30, 31]);

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

    #[test]
    fn test_algorithm() -> Result<()> {
        // let words = "In olden times when wishing still helped one, \
        //              there lived a king whose daughters were all beautiful ; \
        //              and the youngest was so beautiful that the sun itself, \
        //              which has seen so much, was astonished whenever it shone \
        //              in her face.";

        // let words = "The Ministry of Truth, which concerned itself with news, entertainment, education and the fine arts. The Ministry of Peace, which concerned itself with war. The Ministry of Love, which maintained law and order. And the Ministry of Plenty, which was responsible for economic affairs. Their names, in Newspeak: Minitrue, Minipax, Miniluv and Miniplenty.";

        let words = "The hallway smelt of boiled cabbage and old rag mats. At one end of it a coloured poster, too large for indoor display, had been tacked to the wall. It depicted simply an enormous face, more than a metre wide: the face of a man of about forty-five, with a heavy black moustache and ruggedly handsome features. Winston made for the stairs. It was no use trying the lift. Even at the best of times it was seldom working, and at present the electric current was cut off during daylight hours. It was part of the economy drive in preparation for Hate Week. The flat was seven flights up, and Winston, who was thirty-nine and had a varicose ulcer above his right ankle, went slowly, resting several times on the way. On each landing, opposite the lift-shaft, the poster with the enormous face gazed from the wall. It was one of those pictures which are so contrived that the eyes follow you about when you move. BIG BROTHER IS WATCHING YOU, the caption beneath it ran.";

        let ast = Ast::Paragraph(vec![Ast::Text(words.into())]);

        let en_us = Standard::from_embedded(Language::EnglishUS)?;

        let (_, font_manager) = Config::with_title("Test").init()?;
        let config = font_manager.default_config();

        let indentation = Pt(18.0);

        let paragraph = itemize_ast(&ast, &config, Pt(12.0), &en_us, indentation);

        let lines_length = vec![Pt(400.0)];
        let breakpoints = algorithm(&paragraph, &lines_length);
        // let positions = positionate_items(&paragraph.items, &lines_length, &breakpoints);

        let adjustment_ratios = compute_adjustment_ratios_with_breakpoints(
            &paragraph.items,
            &lines_length,
            &breakpoints,
        );

        println!("Line length: {:?}", lines_length[0]);
        println!("Breakpoints: {:?}", breakpoints);
        println!("There are {:?} lines", breakpoints.len());
        print!("\n\n");
        let mut current_line = 0;
        for (c, item) in paragraph.items.iter().enumerate() {
            match item.content {
                Content::BoundingBox(ref glyph) => print!("{}", glyph.glyph),
                Content::Glue { .. } => {
                    if breakpoints.contains(&c) {
                        println!("       [{:?}]", adjustment_ratios[current_line]);
                        current_line += 1;
                    } else {
                        print!(" ");
                    }
                }
                Content::Penalty { .. } => {
                    if breakpoints.contains(&c) {
                        println!("-      [{:?}]", adjustment_ratios[current_line]);
                        current_line += 1;
                    }
                }
            }
        }

        print!("\n\n");

        // panic!("Test");

        Ok(())
    }
}

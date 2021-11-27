//! Utility functions for the various stages of the line breaking algorithm.

use crate::layout::constants::{ADJACENT_LOOSE_TIGHT_PENALTY, MAX_COST, MIN_COST};
use crate::layout::paragraphs::graph::Node;
use crate::layout::paragraphs::utils::paragraphs::get_line_length;

use crate::layout::paragraphs::items::{Content, Item};
use crate::layout::paragraphs::Paragraph;
use printpdf::Pt;
use std::f64;

/// Accumulator to hold the three key related measures.
pub struct Measures {
    /// Measure of the width accumulated so far.
    pub width: Pt,

    /// Measure of the shrinkability accumulated so far.
    pub shrinkability: Pt,

    /// Measure of the stretchability accumulated so far.
    pub stretchability: Pt,
}

/// Computes the adjusment ratio of a line of items, based on their combined
/// width, stretchability and shrinkability. This essentially tells how much
/// effort has to be produce to fit the line to the desired width.
pub fn compute_adjustment_ratio(
    actual_length: Pt,
    desired_length: Pt,
    total_stretchability: Pt,
    total_shrinkability: Pt,
) -> f64 {
    if actual_length == desired_length {
        0.0
    } else if actual_length < desired_length {
        if total_stretchability != Pt(0.0) {
            (desired_length.0 - actual_length.0) / total_stretchability.0
        } else {
            f64::INFINITY
        }
    } else if total_shrinkability != Pt(0.0) {
        (desired_length.0 - actual_length.0) / total_shrinkability.0
    } else {
        f64::INFINITY
    }
}

/// Computes the adjustment ratios of all lines given a set of line lengths and breakpoint indices.
/// This allows to speed up the adaptation of glue items.
pub fn compute_adjustment_ratios_with_breakpoints<'a>(
    items: &[Item<'a>],
    line_lengths: &[Pt],
    breakpoints: &[usize],
) -> Vec<f64> {
    let mut adjustment_ratios: Vec<f64> = Vec::new();

    for (breakpoint_line, breakpoint_index) in breakpoints.iter().enumerate() {
        let desired_length = get_line_length(line_lengths, breakpoint_line);
        let mut actual_length = Pt(0.0);
        let mut line_shrink = Pt(0.0);
        let mut line_stretch = Pt(0.0);
        let next_breakpoint = if breakpoint_line < breakpoints.len() - 1 {
            breakpoints[breakpoint_line + 1]
        } else {
            items.len() - 1
        };

        let beginning = if breakpoint_line == 0 {
            *breakpoint_index
        } else {
            *breakpoint_index + 1
        };

        let range = items
            .iter()
            .enumerate()
            .take(next_breakpoint)
            .skip(beginning);

        for (p, item) in range {
            match item.content {
                Content::BoundingBox { .. } => actual_length += items[p].width,
                Content::Glue {
                    shrinkability,
                    stretchability,
                } => {
                    if p != beginning && p != next_breakpoint {
                        actual_length += item.width;
                        line_shrink += shrinkability;
                        line_stretch += stretchability;
                    }
                }
                Content::Penalty { .. } => {
                    if p == next_breakpoint {
                        actual_length += item.width;
                    }
                }
            }
        }

        adjustment_ratios.push(compute_adjustment_ratio(
            actual_length,
            desired_length,
            line_stretch,
            line_shrink,
        ));
    }

    adjustment_ratios
}

/// Computes the demerits of a line based on its accumulated penalty
/// and badness.
pub fn compute_demerits(penalty: f64, badness: f64) -> f64 {
    if penalty >= 0.0 {
        (1.0 + badness + penalty).powi(2)
    } else if penalty > MIN_COST {
        (1.0 + badness).powi(2) - penalty.powi(2)
    } else {
        (1.0 + badness).powi(2)
    }
}

/// Computes the fitness class of a line based on its adjustment ratio.
pub fn compute_fitness(adjustment_ratio: f64) -> i64 {
    if adjustment_ratio < -0.5 {
        0
    } else if adjustment_ratio < 0.5 {
        1
    } else if adjustment_ratio < 1.0 {
        2
    } else {
        3
    }
}

/// Checks whether or not a given item encodes a forced linebreak.
pub fn is_forced_break<'a>(item: &'a Item<'a>) -> bool {
    match item.content {
        Content::Penalty { value, .. } => value < MIN_COST,
        _ => false,
    }
}

/// Finds all the legal breakpoints within a paragraph. A legal breakpoint
/// is an item index such that this item is either a peanalty which isn't
/// infinite or a glue following a bounding box.
pub fn find_legal_breakpoints(paragraph: &Paragraph) -> Vec<usize> {
    let mut legal_breakpoints: Vec<usize> = vec![0];

    let mut last_item_was_box = false;

    for (i, item) in paragraph.items.iter().enumerate() {
        match item.content {
            Content::Penalty { value, .. } => {
                if value < f64::INFINITY {
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

/// Handles a feasible breakpoint and adds it to the current graph of
/// feasible breakpoints if it's good enough.
#[inline]
pub fn create_node_for_feasible_breakpoint(
    b: usize,
    a: &Node,
    adjustment_ratio: f64,
    item: &Item,
    items: &[Item],
    measures_sum: &Measures,
) -> Node {
    // This is a feasible breakpoint.
    let badness = adjustment_ratio.abs().powi(3);
    let penalty = match item.content {
        Content::Penalty { value, .. } => value,
        _ => 0.0,
    };

    let mut demerits = compute_demerits(penalty, badness);

    // TODO: support double hyphenation penalty.

    // Compute fitness class.
    let fitness = compute_fitness(adjustment_ratio);

    if a.index > 0 && (fitness - a.fitness).abs() > 1 {
        demerits += ADJACENT_LOOSE_TIGHT_PENALTY;
    }

    // TODO: Ignore the width of potential subsequent glue or
    // non-breakable penalty item to avoid rendering glue or
    // penalties at the beginning of lines.
    let measures_to_next_box = get_measures_to_next_box(b, item, items);

    Node {
        index: b,
        line: a.line + 1,
        fitness,
        total_width: measures_sum.width + measures_to_next_box.width,
        total_shrink: measures_sum.shrinkability + measures_to_next_box.shrinkability,
        total_stretch: measures_sum.stretchability + measures_to_next_box.stretchability,
        total_demerits: a.total_demerits + demerits,
    }
}

/// Computes the accumulated measures from the current linebreak
/// to the next bounding box in the provided items.
#[inline]
pub fn get_measures_to_next_box(b: usize, item: &Item, items: &[Item]) -> Measures {
    let mut width_to_next_box = Pt(0.0);
    let mut shrink_to_next_box = Pt(0.0);
    let mut stretch_to_next_box = Pt(0.0);

    for next_item in &items[b..] {
        width_to_next_box += item.width;

        match next_item.content {
            Content::BoundingBox { .. } => break,
            Content::Glue {
                shrinkability,
                stretchability,
            } => {
                shrink_to_next_box += shrinkability;
                stretch_to_next_box += stretchability;
            }
            Content::Penalty { value, .. } => {
                if value >= MAX_COST {
                    break;
                }
            }
        }
    }

    Measures {
        width: width_to_next_box,
        shrinkability: shrink_to_next_box,
        stretchability: stretch_to_next_box,
    }
}

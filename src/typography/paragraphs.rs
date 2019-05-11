//! Mathematical logic for typesetting a sequence of words which have a
//! semantics of "paragraph". That is, the logic to split a sequence of
//! words into lines.

use crate::font::Font;
use crate::saturating::Saturating;
use crate::typography::items::{
    Content, Item, PositionedItem, INFINITELY_NEGATIVE_PENALTY, INFINITELY_POSITIVE_PENALTY,
};
use crate::units::{Mm, Sp, PLUS_INFINITY};
use hyphenation::*;
use num_rational::Ratio;
use num_traits::sign::Signed;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use petgraph::visit::{Bfs, Dfs};
use petgraph::visit::{IntoEdgeReferences, IntoNodeIdentifiers, NodeCompactIndexable};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::vec::Vec;

type Rational = Ratio<Sp>;

fn minus_one() -> Rational {
    Rational::new_raw(Sp(-65536), Sp(1))
}

fn one() -> Rational {
    Rational::new_raw(Sp(65536), Sp(1))
}

fn zero() -> Rational {
    Rational::new_raw(Sp(0), Sp(1))
}

fn one_half() -> Rational {
    Rational::new_raw(Sp(65536), Sp(2))
}

fn plus_infinity() -> Rational {
    Rational::new_raw(PLUS_INFINITY, Sp(1))
}

fn min_adjustment_ratio() -> Rational {
    Rational::new_raw(MIN_ADJUSTMENT_RATIO, Sp(1))
}

fn max_adjustment_ratio() -> Rational {
    Rational::new_raw(MAX_ADJUSTMENT_RATIO, Sp(1))
}

const DASH_GLYPH: char = '-';
const SPACE_WIDTH: Sp = Sp(185771);
const DEFAULT_LINE_LENGTH: Sp = Sp(50135040);
const MIN_COST: i64 = -1000;
const MAX_COST: i64 = 1000;
const ADJACENT_LOOSE_TIGHT_PENALTY: f64 = 50.0;
const MIN_ADJUSTMENT_RATIO: Sp = Sp(-1);
const MAX_ADJUSTMENT_RATIO: Sp = Sp(10);

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
    Item::glue(ideal_spacing, SPACE_WIDTH * 2, SPACE_WIDTH)
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

fn find_beginning_of_line(paragraph: &Paragraph, index: usize) -> usize {
    if index == 0 {
        0
    } else {
        let mut minimum = usize::max_value();

        for (i, item) in paragraph.items[(index - 1)..index].iter().enumerate() {
            match item.content {
                Content::BoundingBox { .. } => {
                    minimum = i;
                    break;
                }
                Content::Penalty { value, .. } => {
                    if value == INFINITELY_NEGATIVE_PENALTY {
                        minimum = i;
                        break;
                    }
                }
                _ => continue,
            }
        }

        if minimum == usize::max_value() {
            index
        } else {
            minimum
        }
    }
}

#[derive(Copy, Clone)]
struct Node {
    /// Index of the item represented by the node, within the paragraph.
    pub index: usize,

    /// Line at which the item lives within the paragraph.
    pub line: usize,

    /// The fitness class of the item represented by the node.
    pub fitness: i64,
    pub total_width: Sp,
    pub total_stretch: Sp,
    pub total_shrink: Sp,
    pub total_demerits: f64,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.index == other.index
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

/// Returns the length of the line of given index, from a list of
/// potential line lengths. If the list is too short, the line
/// length will default to `DEFAULT_LINE_LENGTH`.
fn get_line_length(lines_length: &Vec<Sp>, index: usize) -> Sp {
    if index < lines_length.len() {
        lines_length[index]
    } else {
        if lines_length.len() == 1 {
            lines_length[0]
        } else {
            DEFAULT_LINE_LENGTH
        }
    }
}

/// Computes the demerits of a line based on its accumulated penalty
/// and badness.
fn compute_demerits(penalty: &Rational, badness: &Rational) -> f64 {
    let one = Rational::new_raw(Sp(1), Sp(1));

    if penalty >= &zero() {
        one.sadd(badness).sadd(penalty).to_float().powi(2)
    } else if penalty > &Rational::new_raw(Sp(MIN_COST), Sp(1)) {
        one.sadd(badness).to_float().powi(2) - penalty.to_float().powi(2)
    } else {
        one.sadd(badness).to_float().powi(2)
    }
}

/// Computes the fitness class of a line based on its adjustment ratio.
fn compute_fitness(adjustment_ratio: Rational) -> i64 {
    if adjustment_ratio < -one_half() {
        0
    } else if adjustment_ratio < -one_half() {
        1
    } else if adjustment_ratio < one() {
        2
    } else {
        3
    }
}

fn algorithm(paragraph: &Paragraph, lines_length: &Vec<Sp>) -> Vec<usize> {
    let mut graph = StableGraph::<_, f64>::new();
    let mut sum_width = Sp(0);
    let mut sum_stretch = Sp(0);
    let mut sum_shrink = Sp(0);
    let mut best_adjustment_ratio_above_threshold = Rational::new(PLUS_INFINITY, Sp(1));
    let mut current_maximum_adjustment_ratio = plus_infinity();

    let mut last_best_node: Node;
    let mut last_best_node_index: Option<_> = None;

    // Add an initial active node for the beginning of the paragraph.
    graph.add_node(Node {
        index: 0,
        line: 0,
        fitness: 1,
        total_width: Sp(0),
        total_stretch: Sp(0),
        total_shrink: Sp(0),
        total_demerits: 0.0,
    });

    for (b, item) in paragraph.items.iter().enumerate() {
        if item.width < Sp(0) {
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
                can_break = b > 0
                    && (match paragraph.items[b - 1].content {
                        Content::BoundingBox { .. } => true,
                        _ => false,
                    });

                if !can_break {
                    sum_width += item.width;
                    sum_shrink += shrinkability;
                    sum_stretch += stretchability;
                }
            }
            Content::Penalty { value, .. } => {
                can_break = value < INFINITELY_POSITIVE_PENALTY;
            }
        }

        // println!(
        //     "----- STATS ------\n\
        //      Width sum: {:?}\n\
        //      Shrink sum: {:?}\n\
        //      Stretch sum: {:?}",
        //     sum_width, sum_shrink, sum_stretch
        // );

        if !can_break {
            // println!("Item {} cannot break, skipping to next item.", b);
            continue;
        }

        // Update the set of active nodes.

        let mut last_active_node: Option<&Node> = None;
        let mut feasible_breakpoints: Vec<(Node, NodeIndex)> = Vec::new();
        let mut node_to_remove: Vec<NodeIndex> = Vec::new();

        for node in graph.node_identifiers() {
            if let Some(a) = graph.node_weight(node) {
                println!("[ Adjustment ratio from {} to {} ]", a.index, b);
                let line_shrink = sum_shrink - a.total_shrink;
                let line_stretch = sum_stretch - a.total_stretch;
                let actual_width = sum_width - a.total_width;

                println!("   -> Line shrink: {:?}", line_shrink);
                println!("   -> Line stretch: {:?}", line_stretch);

                let adjustment_ratio = compute_adjustment_ratio(
                    actual_width,
                    get_line_length(&lines_length, a.line),
                    line_stretch,
                    line_shrink,
                );

                println!("   -> Adjustment ratio: {:?}", adjustment_ratio.to_float());
                println!("                  -> {:?}", adjustment_ratio);

                if adjustment_ratio > current_maximum_adjustment_ratio {
                    best_adjustment_ratio_above_threshold =
                        adjustment_ratio.min(best_adjustment_ratio_above_threshold)
                }

                if adjustment_ratio < min_adjustment_ratio() || is_forced_break(item) {
                    // Items from a to b cannot fit on the same line.
                    println!("Removing node {:?}", node);
                    node_to_remove.push(node);
                    last_active_node = Some(a);
                }

                if adjustment_ratio >= min_adjustment_ratio()
                    && adjustment_ratio <= max_adjustment_ratio()
                {
                    // This is a feasible breakpoint.
                    let badness = adjustment_ratio.abs().spow(3);
                    println!("   -> Badness: {:?}", badness.to_float());
                    let penalty = Rational::new(
                        Sp(match item.content {
                            Content::Penalty { value, .. } => value,
                            _ => 0,
                        }),
                        Sp(1),
                    );
                    println!("   -> Penalty: {:?}", penalty.to_float());

                    let mut demerits = compute_demerits(&penalty, &badness);

                    println!("   -> Demerits: {:?}", demerits);

                    // TODO: support double hyphenation penalty.

                    // Compute fitness class.
                    let fitness = compute_fitness(adjustment_ratio);

                    if a.index > 0 && (fitness - a.fitness).abs() > 1 {
                        demerits = demerits + ADJACENT_LOOSE_TIGHT_PENALTY;
                    }

                    // TODO: Ignore the width of potential subsequent glue or
                    // non-breakable penalty item to avoid rendering glue or
                    // penalties at the beginning of lines.

                    let mut width_to_next_box = Sp(0);
                    let mut shrink_to_next_box = Sp(0);
                    let mut stretch_to_next_box = Sp(0);

                    for c in b..paragraph.items.len() {
                        let next_item = &paragraph.items[c];

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

                    let new_node = Node {
                        index: b,
                        line: a.line + 1,
                        fitness,
                        total_width: sum_width + width_to_next_box,
                        total_shrink: sum_shrink + shrink_to_next_box,
                        total_stretch: sum_stretch + stretch_to_next_box,
                        total_demerits: a.total_demerits + demerits,
                    };
                    feasible_breakpoints.push((new_node, node));
                }
            }
        }

        for node in node_to_remove {
            println!("========> Removing node {:?}", node);
            graph.remove_node(node);
        }

        println!(
            "Looking for lowest score breakpoint. There are {} breakpoints.",
            feasible_breakpoints.len()
        );

        // If there is a feasible break at b, then append the best such break
        // as an active node.
        if feasible_breakpoints.len() > 0 {
            let (mut last_best_node, mut last_node_parent_id) = feasible_breakpoints[0];

            for (node, parent_id) in feasible_breakpoints.iter() {
                println!("Node #{:?} with parent #{:?}", node, parent_id);
                if node.total_demerits < last_best_node.total_demerits {
                    last_best_node = *node;
                    last_node_parent_id = *parent_id;
                }
            }

            println!("Best breakpoint node: {:?}", last_best_node);

            let inserted_node = graph.add_node(last_best_node);
            last_best_node_index = Some(inserted_node);

            // Create a precedence relationship between a and the best node.
            graph.add_edge(
                inserted_node,
                last_node_parent_id,
                last_best_node.total_demerits,
            );

            // graph.add_edge(
            //     last_node_parent_id,
            //     inserted_node,
            //     last_best_node.total_demerits.to_integer(),
            // );

            println!(
                "Node added to active graph. Graph has {} nodes.",
                graph.node_count()
            );
        }

        match item.content {
            Content::Glue {
                shrinkability,
                stretchability,
            } => {
                sum_width += item.width;
                sum_shrink = sum_shrink.sadd(&shrinkability);
                sum_stretch = sum_stretch.sadd(&stretchability);
            }
            _ => {}
        }
    }

    // TODO: handle situation where there's no option to fall within the window of
    // accepted adjustment ratios.

    println!("{:?}", Dot::with_config(&graph, &[]));

    // Choose the active node with fewest total demerits.
    let mut best_node: Option<NodeIndex> = None;
    for node in graph.node_identifiers() {
        println!("Current node: {:?}, best node: {:?}", node, best_node);
        if let Some(a) = graph.node_weight(node) {
            match best_node {
                Some(best_node_index) => {
                    if let Some(best_node_value) = graph.node_weight(best_node_index) {
                        if a.total_demerits < best_node_value.total_demerits {
                            println!("Total demerits are better.");
                            best_node = Some(node);
                        } else {
                            println!(
                                "Total demerits aren't better ({:?} vs {:?}).",
                                a.total_demerits, best_node_value.total_demerits
                            );
                        }
                    }
                }

                None => best_node = Some(node),
            }
        }
    }

    // Follow the edges backwards.
    let mut result: Vec<usize> = Vec::new();

    if let Some(best_node_index) = best_node {
        println!(
            "->>>>>>>>>>>>>>> LAST BEST NODE INDEX: {:?}",
            best_node_index
        );
        let mut dfs = Dfs::new(&graph, best_node_index);
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

fn is_forced_break(item: &Item) -> bool {
    match item.content {
        Content::Penalty { value, .. } => value < MIN_COST,
        _ => false,
    }
}

pub trait ToFloat {
    fn to_float(&self) -> f64;
}

impl ToFloat for Rational {
    fn to_float(&self) -> f64 {
        self.numer().0 as f64 / self.denom().0 as f64
    }
}
// fn positionate_items(items: Vec<Item>, lines_length: Vec<i32>, breakpoints: Vec<i32>) -> PositionedItem {
//     let adjustment_ratio = compute_adjustment_ratio(actual_length: Sp, desired_length: Sp, total_stretchability: Sp, total_shrinkability: Sp, )
// }

/// Computes the adjustment ratios of all lines given a set of line lengths and breakpoint indices.
fn compute_adjustment_ratios_with_breakpoints(
    items: &Vec<Item>,
    line_lengths: &Vec<Sp>,
    breakpoints: &Vec<usize>,
) -> Vec<Rational> {
    let mut adjustment_ratios: Vec<Rational> = Vec::new();

    for (breakpoint_line, breakpoint_index) in breakpoints.iter().enumerate() {
        let desired_length = get_line_length(line_lengths, breakpoint_line);
        let mut actual_length = Sp(0);
        let mut line_shrink = Sp(0);
        let mut line_stretch = Sp(0);
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

        for p in beginning..next_breakpoint {
            match items[p].content {
                Content::BoundingBox { .. } => actual_length += items[p].width,
                Content::Glue {
                    shrinkability,
                    stretchability,
                } => {
                    if p != beginning && p != next_breakpoint {
                        actual_length = actual_length.sadd(&items[p].width);
                        line_shrink = line_shrink.sadd(&shrinkability);
                        line_stretch = line_stretch.sadd(&stretchability);
                    }
                }
                Content::Penalty { .. } => {
                    if p == next_breakpoint {
                        actual_length += items[p].width;
                    }
                }
            }
        }

        println!("[ FROM {} to {}]", beginning, next_breakpoint);
        println!("    -> Actual length: {:?}", actual_length);
        println!("    -> Desired length: {:?}", desired_length);
        println!("    -> Line shrink: {:?}", line_shrink);
        println!("    -> Line stretch: {:?}", line_stretch);

        adjustment_ratios.push(compute_adjustment_ratio(
            actual_length,
            desired_length,
            line_stretch,
            line_shrink,
        ));
    }

    adjustment_ratios
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
) -> Rational {
    if actual_length == desired_length {
        zero()
    } else if actual_length < desired_length {
        if total_stretchability != Sp(0) {
            println!(
                "{:?} / {:?}",
                desired_length - actual_length,
                total_stretchability
            );
            Rational::new(desired_length - actual_length, total_stretchability)
        } else {
            plus_infinity()
        }
    } else {
        if total_shrinkability != Sp(0) {
            Rational::new(desired_length - actual_length, total_shrinkability)
        } else {
            plus_infinity()
        }
    }
}

/// Generates a list of positioned items from a list of items making up a paragraph.
/// The generated list is ready to be rendered.
fn positionate_items(
    items: Vec<Item>,
    line_lengths: Vec<Sp>,
    breakpoints: Vec<usize>,
) -> Vec<PositionedItem> {
    let adjustment_ratios =
        compute_adjustment_ratios_with_breakpoints(&items, &line_lengths, &breakpoints);
    let mut positioned_items: Vec<PositionedItem> = Vec::new();

    for (breakpoint_line, breakpoint_index) in breakpoints.iter().enumerate() {
        let adjustment_ratio = adjustment_ratios[breakpoint_line].max(min_adjustment_ratio());
        let mut horizontal_offset = zero();
        let beginning = if breakpoint_line == 0 {
            *breakpoint_index
        } else {
            *breakpoint_index + 1
        };

        for p in beginning..breakpoints[breakpoint_line + 1] {
            match items[p].content {
                Content::BoundingBox { .. } => positioned_items.push(PositionedItem {
                    index: p,
                    line: breakpoint_line,
                    horizontal_offset: horizontal_offset.round().to_integer(),
                    width: items[p].width,
                }),
                Content::Glue {
                    shrinkability,
                    stretchability,
                } => {
                    if p != beginning && p != breakpoints[breakpoint_line + 1] {
                        let width = Rational::new(items[p].width, Sp(1));

                        let gap: Rational = if adjustment_ratio < zero() {
                            width + adjustment_ratio * Rational::new(shrinkability, Sp(1))
                        } else {
                            width + adjustment_ratio * Rational::new(stretchability, Sp(1))
                        };

                        // TODO: add an option to handle the inclusion of glue.

                        horizontal_offset = horizontal_offset + gap;
                    }
                }
                Content::Penalty { .. } => {
                    if p == breakpoints[breakpoint_line + 1] && items[p].width > Sp(0) {
                        positioned_items.push(PositionedItem {
                            index: p,
                            line: breakpoint_line,
                            horizontal_offset: horizontal_offset.round().to_integer(),
                            width: items[p].width,
                        })
                    }
                }
            }
        }
    }

    positioned_items
}

/// Unit tests for the paragraphs typesetting.
#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::typography::items::Content;
    use crate::typography::paragraphs::{
        algorithm, compute_adjustment_ratios_with_breakpoints, find_legal_breakpoints,
        itemize_paragraph, ToFloat,
    };
    use crate::units::{Mm, Sp};
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
        assert_eq!(paragraph.items.len(), 32);

        // Indentated paragraph, implying the presence of a leading empty box.
        let paragraph = itemize_paragraph(words, Sp(120_000), &font, 12.0, &en_us);
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
        let paragraph = itemize_paragraph(words, Sp(120_000), &font, 12.0, &en_us);

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

    #[test]
    fn test_algorithm() -> Result<()> {
        let words = "In olden times when wishing still helped one, \
                     there lived a king whose daughters were all beautiful ; \
                     and the youngest was so beautiful that the sun itself, \
                     which has seen so much, was astonished whenever it shone \
                     in her face.";

        // let words = "The Ministry of Truth, which concerned itself with news, entertainment, education and the fine arts. The Ministry of Peace, which concerned itself with war. The Ministry of Love, which maintained law and order. And the Ministry of Plenty, which was responsible for economic affairs. Their names, in Newspeak: Minitrue, Minipax, Miniluv and Miniplenty.";

        let en_us = Standard::from_embedded(Language::EnglishUS)?;

        let (_, font_manager) = Config::with_title("Test").init()?;

        let regular_font_name = "CMU Serif Roman";
        // let bold_font_name = "CMU Serif Bold";

        let font = font_manager
            .get(regular_font_name)
            .ok_or(Error::FontNotFound(PathBuf::from(regular_font_name)))?;

        let indentation = Sp::from(Mm(5.0));

        let paragraph = itemize_paragraph(words, indentation, &font, 12.0, &en_us);

        let lines_length = vec![];
        let breakpoints = algorithm(&paragraph, &lines_length);
        // let positions = positionate_items(paragraph.items, lines_length, breakpoints);

        let adjustment_ratios = compute_adjustment_ratios_with_breakpoints(
            &paragraph.items,
            &lines_length,
            &breakpoints,
        );

        println!("Breakpoints: {:?}", breakpoints);
        let mut current_line = 0;
        for (c, item) in paragraph.items.iter().enumerate() {
            match item.content {
                Content::BoundingBox { glyph } => print!("{}", glyph),
                Content::Glue { .. } => {
                    if breakpoints.contains(&c) {
                        print!("       [{:?}]", adjustment_ratios[current_line].to_float());
                        print!("\n");
                        current_line += 1;
                    } else {
                        print!(" ");
                    }
                }
                Content::Penalty { .. } => {
                    if breakpoints.contains(&c) {
                        print!("-      [{:?}]", adjustment_ratios[current_line].to_float());
                        print!("\n");
                        current_line += 1;
                    }
                }
            }
        }

        panic!("Test");

        Ok(())
    }
}

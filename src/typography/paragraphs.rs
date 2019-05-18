//! Mathematical logic for typesetting a sequence of words which have a
//! semantics of "paragraph". That is, the logic to split a sequence of
//! words into lines.

use std::cmp::Ordering;
use std::collections::hash_map::{Entry, HashMap};
use std::f64;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::slice::Iter;

use hyphenation::*;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::Dfs;
use petgraph::visit::IntoNodeIdentifiers;
use printpdf::Pt;

use crate::font::{FontConfig, FontStyle};
use crate::parser::ast::Ast;
use crate::typography::items::{Content, Item, PositionedItem};
use crate::typography::Glyph;

const DASH_GLYPH: char = '-';
const SPACE_WIDTH: Pt = Pt(5.0);
const DEFAULT_LINE_LENGTH: Pt = Pt(680.0);
const MIN_COST: f64 = -1000.0;
const MAX_COST: f64 = 1000.0;
const ADJACENT_LOOSE_TIGHT_PENALTY: f64 = 50.0;
const MIN_ADJUSTMENT_RATIO: f64 = -1.0;
const MAX_ADJUSTMENT_RATIO: f64 = 10.0;
const PLUS_INFINITY: Pt = Pt(f64::INFINITY);

/// The ideal spacing between two words.
pub const IDEAL_SPACING: Pt = Pt(5.0);

/// Holds a list of items describing a paragraph.
#[derive(Debug, Default)]
pub struct Paragraph<'a> {
    /// Sequence of items representing the structure of the paragraph.
    pub items: Vec<Item<'a>>,
}

impl<'a> Paragraph<'a> {
    /// Instantiates a new paragraph.
    pub fn new() -> Paragraph<'a> {
        Paragraph { items: Vec::new() }
    }

    /// Pushes an item at the end of the paragraph.
    pub fn push(&mut self, item: Item<'a>) {
        self.items.push(item)
    }

    /// Returns an iterator to the items of the paragraph.
    pub fn iter(&self) -> Iter<Item> {
        self.items.iter()
    }
}

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
        Ast::Title { level, content } => {
            let size = size + Pt(3.0 * ((4 - *level as isize).max(1)) as f64);
            itemize_ast_aux(
                content,
                font_config,
                size,
                dictionary,
                current_style.bold(),
                buffer,
            );
            buffer.push(Item::glue(Pt(0.0), PLUS_INFINITY, Pt(0.0)));
            buffer.push(Item::penalty(Pt(0.0), f64::NEG_INFINITY, false));
        }

        Ast::Bold(content) => {
            itemize_ast_aux(
                content,
                font_config,
                size,
                dictionary,
                current_style.bold(),
                buffer,
            );
        }

        Ast::Italic(content) => {
            itemize_ast_aux(
                content,
                font_config,
                size,
                dictionary,
                current_style.italic(),
                buffer,
            );
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

        Ast::Group(children) => {
            for child in children {
                itemize_ast_aux(child, font_config, size, dictionary, current_style, buffer);
            }
        }

        Ast::Paragraph(children) => {
            for child in children {
                itemize_ast_aux(child, font_config, size, dictionary, current_style, buffer);
            }

            // Appends two items to ensure the end of any paragraph is treated properly: a glue
            // specifying the available space at the right of the last tine, and a penalty item to
            // force a line break.
            buffer.push(Item::glue(Pt(0.0), PLUS_INFINITY, Pt(0.0)));
            buffer.push(Item::penalty(Pt(0.0), f64::NEG_INFINITY, false));
        }

        _ => (),
    }
}

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
fn glue_from_context(_previous_glyph: Option<Glyph>, ideal_spacing: Pt) -> Item {
    // Todo: make this glue context dependent.
    Item::glue(ideal_spacing, SPACE_WIDTH, SPACE_WIDTH * 0.5)
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

/// Aggregates various measures up to and from a feasible breakpoint.
#[derive(Copy, Clone)]
struct Node {
    /// Index of the item represented by the node, within the paragraph.
    pub index: usize,

    /// Line at which the item lives within the paragraph.
    pub line: usize,

    /// The fitness class of the item represented by the node.
    pub fitness: i64,

    /// Total width from the previous breakpoint to this one.
    pub total_width: Pt,

    /// Total stretchability from the previous breakpoint to this one.
    pub total_stretch: Pt,

    /// Total shrinkability from the previous breakpoint to this one.
    pub total_shrink: Pt,

    /// Accumulated demerits from previous breakpoints.
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
fn get_line_length(lines_length: &[Pt], index: usize) -> Pt {
    if index < lines_length.len() {
        lines_length[index]
    } else {
        *lines_length.first().unwrap_or(&DEFAULT_LINE_LENGTH)
    }
}

/// Computes the demerits of a line based on its accumulated penalty
/// and badness.
fn compute_demerits(penalty: f64, badness: f64) -> f64 {
    if penalty >= 0.0 {
        (1.0 + badness + penalty).powi(2)
    } else if penalty > MIN_COST {
        (1.0 + badness).powi(2) - penalty.powi(2)
    } else {
        (1.0 + badness).powi(2)
    }
}

/// Computes the fitness class of a line based on its adjustment ratio.
fn compute_fitness(adjustment_ratio: f64) -> i64 {
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

/// Finds the optimal sequence of breakpoints that minimize
/// the amount of demerits while breaking a paragraph down
/// into lines.
///
/// It returns the indexes of items which have been chosen as
/// breakpoints.
#[allow(clippy::cyclomatic_complexity)]
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

                if adjustment_ratio >= MIN_ADJUSTMENT_RATIO
                    && adjustment_ratio <= MAX_ADJUSTMENT_RATIO
                {
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

                    let mut width_to_next_box = Pt(0.0);
                    let mut shrink_to_next_box = Pt(0.0);
                    let mut stretch_to_next_box = Pt(0.0);

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

/// Checks whether or not a given item encodes a forced linebreak.
fn is_forced_break<'a>(item: &'a Item<'a>) -> bool {
    match item.content {
        Content::Penalty { value, .. } => value < MIN_COST,
        _ => false,
    }
}

/// Computes the adjustment ratios of all lines given a set of line lengths and breakpoint indices.
/// This allows to speed up the adaptation of glue items.
fn compute_adjustment_ratios_with_breakpoints<'a>(
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

/// Computes the adjusment ratio of a line of items, based on their combined
/// width, stretchability and shrinkability. This essentially tells how much
/// effort has to be produce to fit the line to the desired width.
fn compute_adjustment_ratio(
    actual_length: Pt,
    desired_length: Pt,
    total_stretchability: Pt,
    total_shrinkability: Pt,
) -> f64 {
    if actual_length == desired_length {
        0.0
    } else if actual_length < desired_length {
        if total_stretchability != Pt(0.0) {
            ((desired_length.0 - actual_length.0) / total_stretchability.0)
        } else {
            f64::INFINITY
        }
    } else if total_shrinkability != Pt(0.0) {
        ((desired_length.0 - actual_length.0) / total_shrinkability.0)
    } else {
        f64::INFINITY
    }
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

    use hyphenation::*;
    use printpdf::Pt;

    use crate::config::Config;
    use crate::parser::ast::Ast;
    use crate::typography::items::Content;
    use crate::typography::paragraphs::{
        algorithm, compute_adjustment_ratios_with_breakpoints, find_legal_breakpoints, itemize_ast,
    };
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

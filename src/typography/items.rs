//! Various blocks holding information and specifications about the structure
//! of a paragraph.
use crate::font::Font;
use crate::units::{Pt, Sp};

/// Value of the most negative penalty possible. This is considered infinite.
pub const INFINITELY_NEGATIVE_PENALTY: i64 = i64::min_value();

/// Value of the most positive penalty possible. This is considered infinite. g
pub const INFINITELY_POSITIVE_PENALTY: i64 = i64::max_value();

/// Top abstraction of an item, which is a specification for a box, a glue
/// or a penalty.
#[derive(Debug)]
pub struct Item {
    /// The width of the item in scaled units.
    pub width: Sp,

    /// The type of the item.
    pub content: Content,
}

/// Possible available types for an item.
#[derive(Debug)]
pub enum Content {
    /// A bounding box refers to something that is meant to be typeset.
    ///
    /// Though it holds the glyph it's representing, this item is
    /// essentially a black box as the only revelant information
    /// about it for splitting a paragraph into lines is its width.
    BoundingBox {
        /// The glyph that is meant to be typeset.
        glyph: char,
    },
    /// Glue is a blank space which can see its width altered in specified ways.
    ///
    /// It can either stretch or shrink up to a certain limit, and is used as
    /// mortar to leverage to reach a target column width.
    Glue {
        /// How inclined the glue is to stretch from its natural width, in scaled points.
        stretchability: Sp,

        /// How inclined the glue is to shrink from its natural width, in scaled points.
        shrinkability: Sp,
    },
    /// Penalty is a potential place to end a line and step to another. It's helpful
    /// to cut a line in the middle of a word (hyphenation) or to enforce a break
    /// at the end of paragraphs.
    Penalty {
        /// The "cost" of the penalty.
        value: i64,

        /// Whether or not the penalty is considered as flagged.
        flagged: bool,
    },
}

impl Item {
    /// Creates a box for a particular glyph and font.
    pub fn from_glyph(glyph: char, font: &Font, font_size: f64) -> Item {
        Item {
            width: Sp::from(Pt(font.char_width(glyph, font_size))),
            content: Content::BoundingBox { glyph },
        }
    }

    /// Creates a bounding box from its width in scaled points and its glyph.
    pub fn bounding_box(width: Sp, glyph: char) -> Item {
        Item {
            width,
            content: Content::BoundingBox { glyph },
        }
    }

    /// Creates some glue.
    pub fn glue(ideal_spacing: Sp, stretchability: Sp, shrinkability: Sp) -> Item {
        Item {
            width: ideal_spacing,
            content: Content::Glue {
                stretchability,
                shrinkability,
            },
        }
    }

    /// Creates a penalty.
    pub fn penalty(width: Sp, value: i64, flagged: bool) -> Item {
        Item {
            width,
            content: Content::Penalty { value, flagged },
        }
    }
}

/// Holds the information of an item that's ready to be rendered.
pub struct PositionedItem {
    // The index of the item within the list of items that make up
    // the paragraph in which is stands.
    index: usize,

    // The index of the line on which this item is to be rendered.
    line: usize,

    // The horizontal offset of the item.
    horizontal_offset: Sp,

    // The (potentially adjusted) width this item should be rendered with.
    width: Sp,
}

//! Various blocks holding information and specifications about the structure
//! of a paragraph.
use crate::font::Font;
use crate::units::{Pt, Sp};

/// Top abstraction of an item, which is a specification for a box, a glue
/// or a penalty.
#[derive(Debug)]
pub struct Item {
    /// The width of the item in scaled units.
    width: Sp,

    /// The type of the item.
    content: Content,
}

/// Possible available types for an item.
#[derive(Debug)]
pub enum Content {
    /// A bounding box refers to something that is meant to be typeset.
    ///
    /// It is essentially a black box as the only revelant information about it
    /// is its width.
    BoundingBox {
        /// The glyph that is meant to be typeset.
        glyph: char,
    },
    /// Glue is a blank space which can see its width altered in specified ways.
    Glue {
        /// How inclined the glue is to stretch, in scaled points.
        stretchability: Sp,

        /// How inclined the glue is to shrink, in scaled points.
        shrinkability: Sp,
    },
    /// Penalty is a potential place to end a line and step to another.
    Penalty {
        /// The "cost" of the penalty.
        value: i32,

        /// Whether or not the penalty is considered as flagged.
        flagged: bool,
    },
}

impl Item {
    /// Creates a box for a particular glyph.
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
    pub fn penalty(width: Sp, value: i32, flagged: bool) -> Item {
        Item {
            width,
            content: Content::Penalty { value, flagged },
        }
    }
}

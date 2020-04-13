//! Various containers.
use printpdf::Pt;

/// Paragraph content items.
pub enum Content {
    /// An horizontal bounding box.
    HorizontalBox {
        /// The width of the bounding box.
        width: Pt,

        /// The height of the bounding box.
        height: Pt,
    },
    /// A stretchable/shrinkable horizontal box.
    Glue {
        /// The height of the glue box.
        height: Pt,

        /// The maximum horizontal length the glue box can shrink
        /// from its original width.
        shrinkability: Pt,

        /// The maximum horizontal length the glue box can stretch
        /// from its original width.
        stretchability: Pt,
    },
    /// A penalty marker to force some behaviors on the engine. A penalty
    /// symbolizes how bad it would be to break down a paragraph at its
    /// position.
    Penalty {
        /// The value of the penalty if the paragraph were to be broken
        /// down at this penalty.
        value: f64,
    },
}

/// Vertical item.
pub struct VerticalItem {}

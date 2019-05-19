use printpdf::Pt;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Aggregates various measures up to and from a feasible breakpoint.
#[derive(Copy, Clone)]
pub struct Node {
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

//! Various constants used for laying out the items of a document.

use printpdf::Pt;
use std::f64;

// Linebreaking constants.
/// The glyph that represents a char.
// FIXME: replace this with an instance of `Glyph`.
pub const DASH_GLYPH: char = '-';

/// The width a whitespace.
pub const SPACE_WIDTH: Pt = Pt(5.0);

/// The default length of a line if no desired length is specified.
pub const DEFAULT_LINE_LENGTH: Pt = Pt(680.0);

/// The minimal cost of a penalty to count as a legal breakpoint.
pub const MIN_COST: f64 = -1000.0;

/// The maximal cost of a penalty to count as a legal breakpoint.
pub const MAX_COST: f64 = 1000.0;

/// The additional cost that should be added to a penalty when the engine
/// picks up to adjacent hyphens.
pub const ADJACENT_LOOSE_TIGHT_PENALTY: f64 = 50.0;

/// Minimum adjustment ratio to consider a breakpoint is legal.
pub const MIN_ADJUSTMENT_RATIO: f64 = -1.0;

/// Maximal adjustment ratio to consider a breakpoint is legal.
pub const MAX_ADJUSTMENT_RATIO: f64 = 10.0;

/// An infinite length in points.
pub const PLUS_INFINITY: Pt = Pt(f64::INFINITY);

/// The ideal spacing between two words.
pub const IDEAL_SPACING: Pt = Pt(5.0);

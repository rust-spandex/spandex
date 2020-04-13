//! The errors relative to layouts.
use crate::layout::pages::Page;

/// A structure holding the information necessary to properly
/// locate a configuration error.
#[derive(Debug)]
pub struct Error {
    page: Page,
}

/// Detailed configuration errors.
#[derive(Debug)]
pub enum ConfigError {
    /// Case where a page is being created without any column in it.
    PageWithoutColumn(Error),
}

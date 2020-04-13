//! Pages are the top level abstraction of a document. They break it down
//! into smaller units, and are themselves broken down into columns.

use std::fmt;
use crate::layout::pages::columns::Column;

pub struct Page {
    /// The number of the page within the document.
    pub number: i32,
    
    /// A sorted array of columns the page is broken down into. Arriving at
    /// the end of a column in this array leads to the next column within
    /// the same array. When the last column is reached, the next column
    /// is that of the next page.
    pub columns: Vec<Column>,

    /// The index of the current column.
    current_column_index: usize
}

impl Page {
    pub fn get_current_column(self: &Self) -> Option<Column> {
        if current_column_index >= self.columns.len() {
            None
        }

        Some(self.columns[self.current_column_index]);
    }

    pub fn get_next_column(self: &Self) -> Option<Column> {
        *self.current_column_index++;
        self.get_current_column()
    }

    pub fn add_column(self: &Self, column: &Column) {
        self.columns.push(column);
    }
}

/// Unit tests for the paragraphs typesetting.
#[cfg(test)]
mod tests {
    use printpdf::Pt;
    use crate::layout::pages::Page;

    use crate::Result;

    #[test]
    fn test_legal_breakpoints() -> Result<()> {
        let page = Page {
            number: 1,
            columns: Vec<Column>::new(),
            current_column_index: 0
        };

        page.add_column(Column {
            width: Pt(400.0),
            height: Pt(200.0),
            current_vertical_position: Pt(0.0)
        });

        page.add_column(Column {
            width: Pt(370.0),
            height: Pt(200.0),
            current_vertical_position: Pt(0.0)
        });

        assert_eq!(page.columns.len(), 1);
        
        assert!(page.get_current_column().is_some())
        
        let next_column = page.get_next_column();
        assert!(page.get_current_column().is_some())
        
        assert!(page.get_next_column().is_none())

        Ok(())
    }
}
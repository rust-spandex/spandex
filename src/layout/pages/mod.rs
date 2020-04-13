pub mod columns;
pub mod items;

use crate::layout::pages::columns::Column;

/// Holds the information pertaining to a page in the document.
#[derive(Debug, Clone)]
pub struct Page {
    /// The number of the page within the document.
    pub number: i32,

    /// A sorted array of columns the page is broken down into. Arriving at
    /// the end of a column in this array leads to the next column within
    /// the same array. When the last column is reached, the next column
    /// is that of the next page.
    pub columns: Vec<Column>,

    /// The index of the current column.
    current_column_index: usize,
}

impl Page {
    /// Returns a new page capable of holding a specified amount of columns.
    pub fn new(columns: Vec<Column>, number: i32) -> Page {
        if columns.len() == 0 {
            // Todo: use ConfigError.
            panic!("Bad layout: every page must have at least one column.");
        }
        Page {
            number,
            columns,
            current_column_index: 0,
        }
    }

    /// Returns the first column of the page.
    pub fn get_first_column(self: &Self) -> Column {
        self.columns[0]
    }

    /// Returns the i-th column from the column in the page.
    pub fn get_ith_column_from_current(self: &Self, i: usize) -> Option<Column> {
        let target_column_index = self.current_column_index + i;
        if target_column_index < self.columns.len() {
            Some(self.columns[target_column_index])
        } else {
            None
        }
    }

    /// Returns the current column of the page. If the page was
    /// already walked through, `None` is returned.
    pub fn get_current_column(self: &Self) -> Column {
        self.columns[self.current_column_index]
    }

    /// Returns the column that comes directly after the current
    /// column of the page. If the current column is the last
    /// page's column, returns `None`.
    pub fn get_next_column(self: &mut Self) -> Option<Column> {
        self.current_column_index += 1;

        if self.current_column_index < self.columns.len() {
            Some(self.get_current_column())
        } else {
            None
        }
    }

    /// Appends the provided column to the page.
    pub fn add_column(self: &mut Self, column: Column) {
        self.columns.push(column);
    }
}

/// Unit tests for the page structure.
#[cfg(test)]
mod tests {
    use crate::layout::pages::columns::Column;
    use crate::layout::pages::Page;
    use printpdf::Pt;

    use crate::Result;

    #[test]
    fn test_pages() -> Result<()> {
        let mut page = Page {
            number: 1,
            columns: Vec::new(),
            current_column_index: 0,
        };

        page.add_column(Column::new(Pt(0.0), Pt(0.0), Pt(400.0), Pt(200.0)));
        page.add_column(Column::new(Pt(0.0), Pt(0.0), Pt(370.0), Pt(200.0)));

        assert_eq!(page.columns.len(), 2);

        let _ = page.get_next_column();

        assert!(page.get_next_column().is_none());

        Ok(())
    }
}

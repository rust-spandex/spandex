//! Holds the information about a column on some page of the document.
//! A column is the second level of abstraction of a document, as it
//! breaks down a page into smaller units.

use printpdf::Pt;

/// A 2-dimensional point (x, y).
pub struct Point(pub Pt, pub Pt);

/// A column is rectangle with specified width and height. It can hold
/// different kinds of content and is used by the layout to know what
/// can be rendered and where.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Column {
    /// The width of the column in points. This represents the amount of
    /// horizontal space a lower level abstraction is allowed to spread
    /// when part of this column.
    pub width: Pt,

    /// The height of the column in points.
    pub height: Pt,

    /// The current vertical position of the caret within the column.
    pub current_vertical_position: Pt,

    /// List of the positioned items that should be rendered when
    /// the column gets rendered.
    // pub positioned_items: Vec<PositionedItem>,

    /// The horizontal position of the column's top left corner.
    pub x: Pt,

    /// The vertical position of the column's top left corner.
    pub y: Pt,
}

impl Column {
    /// Creates a new column from its top left corner, its width and height.
    pub fn new(x: Pt, y: Pt, width: Pt, height: Pt) -> Column {
        Column {
            x,
            y,
            width,
            height,
            current_vertical_position: Pt(0.0),
        }
    }

    /// Translates the cursor vertically by the specified offset.
    pub fn move_cursor(self: &mut Self, y: Pt) {
        self.current_vertical_position = y;

        println!("Column cursor now at {:?}", self.current_vertical_position);
    }

    /// Divides a column into subsequent columns.
    pub fn handle_overlap(a: Column, b: Column) -> Vec<Column> {
        if a.y >= b.y + b.height || a.x >= b.x + b.width {
            // a is completely underneath b; do not divide b.
            return vec![b];
        }

        // When A is north west of B and both are overlapping.
        if a.x < b.x && a.y < b.y && a.x + a.width > b.x && a.y + a.height > b.y {
            if a.y + a.height <= b.y {
                // A is above B.
                return vec![b];
            }

            if a.y > b.y {
                // A is west of B.
                let c1 = Column::new(b.x, b.y, b.width, a.y - b.y);

                let c2 = Column::new(a.x + a.width, a.y, b.width - a.width + b.x - a.x, a.height);

                if a.y + a.height < b.y + b.height {
                    let c3 = Column::new(
                        b.x,
                        a.y + a.height,
                        b.width,
                        b.height - c1.height - c2.height,
                    );

                    return vec![c1, c2, c3];
                }

                return vec![c1, c2];
            } else {
            }

            let c1 = Column::new(
                a.x + a.width,
                b.y,
                b.x + b.width - a.x - a.width,
                b.y + b.height - a.y - a.height,
            );

            let c2 = Column::new(
                b.x,
                a.y + a.height,
                b.width,
                b.y + b.height - a.y - a.height,
            );

            return vec![c1, c2];
        }

        // Case when A is only west of B and effectively overlapping.

        if a.x <= b.x && b.x <= a.x + a.width {
            let c1 = Column::new(
                if a.x + a.width > b.x {
                    a.x + a.width
                } else {
                    b.x
                },
                b.y,
                b.width,
                a.y - b.y,
            );

            if a.y + a.height <= b.y + b.height {
                let c2;
                if a.x + a.width < b.x + b.width {
                    c2 = Column::new(a.x + a.width, a.y, b.width - a.width + b.x - a.x, a.height);
                } else {
                    c2 = Column::new(b.x, a.y, a.x - b.x, a.height);
                }

                // Column underneath a.
                let c3 = Column::new(
                    b.x,
                    a.y + a.height,
                    b.width,
                    b.y + b.height - a.height - a.y,
                );

                if a.x >= b.x && a.x <= b.x + b.width {
                    // Column to the left of a.
                    let c22 = Column::new(b.x, a.y, a.x - b.x, a.height);

                    return vec![c1, c2, c22, c3];
                }

                vec![c1, c2, c3]
            } else {
                let c2 = Column::new(b.x, a.y, a.x - b.x, b.y + b.height - a.y);

                vec![c1, c2]
            }
        } else {
            vec![b]
        }
    }
}

#[allow(unused_macros)]
macro_rules! assert_eq_column {
    ($column1: expr, $column2: expr) => {
        assert!(($column1.x.0 - $column2.x.0).abs() < 0.001);
        assert!(($column1.y.0 - $column2.y.0).abs() < 0.001);
        assert!(($column1.width.0 - $column2.width.0).abs() < 0.001);
        assert!(($column1.height.0 - $column2.height.0).abs() < 0.001);
    };
}

#[allow(dead_code)]
const EPSILON: f64 = 0.001;

#[allow(unused_macros)]
macro_rules! assert_eq_vec_column {
    ($columns1: expr, $columns2: expr) => {
        assert_eq!($columns1.len(), $columns2.len());
        for (column1, column2) in $columns1.iter().zip($columns2.iter()) {
            assert!((column1.x.0 - column2.x.0).abs() < 0.001);
            assert!((column1.x.0 - column2.x.0).abs() < 0.001);
            assert!((column1.width.0 - column2.width.0).abs() < 0.001);
            assert!((column1.height.0 - column2.height.0).abs() < 0.001);
        }
    };
}

/// Unit tests for the [Column](crate::layout::pages::columns::Column) structure.
#[cfg(test)]
mod tests {
    use crate::layout::pages::columns::Column;
    use printpdf::Pt;

    use crate::Result;

    #[test]
    fn handle_overlap_top_left_no_horizontal_offset() -> Result<()> {
        let a = Column::new(Pt(0.0), Pt(0.0), Pt(100.0), Pt(100.0));
        let b = Column::new(Pt(0.0), Pt(50.0), Pt(150.0), Pt(150.0));

        let overlap_result = Column::handle_overlap(a, b);

        let _c1 = Column::new(Pt(100.0), Pt(50.0), Pt(50.0), Pt(50.0));
        let _c2 = Column::new(Pt(0.0), Pt(100.0), Pt(150.0), Pt(100.0));
        println!("{:?}", overlap_result);
        // assert_eq_vec_column!(overlap_result, vec![c1, c2]);

        Ok(())
    }

    #[test]
    fn handle_overlap_top_left_with_horizontal_offset() -> Result<()> {
        let a = Column::new(Pt(10.0), Pt(0.0), Pt(100.0), Pt(100.0));
        let b = Column::new(Pt(25.0), Pt(50.0), Pt(150.0), Pt(150.0));

        let _overlap_result = Column::handle_overlap(a, b);

        let _c1 = Column::new(Pt(100.0), Pt(50.0), Pt(50.0), Pt(50.0));
        let _c2 = Column::new(Pt(25.0), Pt(100.0), Pt(150.0), Pt(100.0));
        // assert_eq_vec_column!(overlap_result, vec![c1, c2]);

        Ok(())
    }

    #[test]
    fn handle_overlap_left_middle_with_offset() -> Result<()> {
        let a = Column::new(Pt(10.0), Pt(30.0), Pt(30.0), Pt(10.0));
        let b = Column::new(Pt(25.0), Pt(10.0), Pt(50.0), Pt(40.0));

        let _overlap_result = Column::handle_overlap(a, b);

        let _c1 = Column::new(Pt(25.0), Pt(10.0), Pt(50.0), Pt(20.0));
        let _c2 = Column::new(Pt(40.0), Pt(30.0), Pt(35.0), Pt(10.0));
        let _c3 = Column::new(Pt(25.0), Pt(40.0), Pt(50.0), Pt(10.0));
        // assert_eq_vec_column!(overlap_result, vec![c1, c2, c3]);
        // assert_eq_column!(overlap_result[0], c1);
        // assert_eq_column!(overlap_result[1], c2);
        // assert_eq_column!(overlap_result[2], c3);

        Ok(())
    }
}

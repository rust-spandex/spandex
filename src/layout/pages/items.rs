use printpdf::Pt;

pub enum Content {
    HorizontalBox {
        width: Pt,
        height: Pt,
    },
    Glue {
        height: Pt,
        shrinkability: Pt,
        stretchability: Pt,
    },
    Penalty {
        value: f64,
    },
}

pub struct VerticalItem {
    content: Content,
}

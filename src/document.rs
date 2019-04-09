//! This module allows to create beautiful documents.

use printpdf::{PdfDocument, PdfDocumentReference, PdfPageReference, PdfLayerReference, Mm, Pt};

/// Converts Pts to Mms.
pub fn mm(pts: i64) -> Mm {
    Into::<Mm>::into(Pt(pts as f64))
}

/// This struct contains the pdf document.
pub struct Document {

    /// The inner document from printpdf.
    document: PdfDocumentReference,

    /// The current page.
    page: PdfPageReference,

    /// The current layer.
    layer: PdfLayerReference,

}

impl Document {
    /// Creates a new pdf document from its name and its size in pt.
    pub fn new(name: &str, width: i64, height: i64) -> Document {

        let (document, page, layer) = PdfDocument::new(name, mm(width), mm(height), "");
        let page = document.get_page(page);
        let layer = page.get_layer(layer);

        Document {
            document,
            page,
            layer,
        }

    }

    /// Returns a reference to the inner pdf document.
    pub fn inner(&self) -> &PdfDocumentReference {
        &self.document
    }

    /// Returns a mutable reference to the inner pdf document.
    pub fn inner_mut(&mut self) -> &mut PdfDocumentReference {
        &mut self.document
    }
}

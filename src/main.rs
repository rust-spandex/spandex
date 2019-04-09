use spandex::document::{Document, pt};
use spandex::font::FontManager;

fn main() {
    let mut document = Document::new("Hello", pt(210.0), pt(297.0));

    let font_manager = FontManager::init(&mut document)
        .expect("Failed to create font manager");

    let font = font_manager.get("CMU Serif Roman")
        .expect("Failed to get font");

    document.write_content(include_str!("../assets/a-la-recherche-du-temps-perdu.txt"), font, 10.0);

    document.save("output.pdf");
}

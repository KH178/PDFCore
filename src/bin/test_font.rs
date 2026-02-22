fn main() {
    let bytes = include_bytes!("../../examples/browser/assets/Roboto-Regular.ttf");
    println!("Embedded bytes size: {}", bytes.len());
    match ai_pdf_writer::core::font::Font::from_bytes(bytes.to_vec(), "Roboto-Regular".to_string()) {
        Ok(_) => println!("Font loaded successfully!"),
        Err(e) => println!("Error loading font: {}", e),
    }
}

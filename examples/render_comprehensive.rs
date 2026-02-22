use ai_pdf_writer::core::template::Template;
use ai_pdf_writer::core::document::Document;
use ai_pdf_writer::core::font::Font as CoreFont;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Comprehensive PDF Layout...");

    // 1. Load the pdfCoret package
    let mut template = Template::from_zip("examples/comprehensive_test.pdfCoret")?;
    println!("Template parsed successfully. Root: {:?}", template.root);

    // 3. Create Document
    let mut doc = Document::new();

    // 4. Load a standard font
    let font_bytes = include_bytes!("browser/assets/Roboto-Regular.ttf");
    let font = CoreFont::from_bytes(font_bytes.to_vec(), "Roboto-Regular".to_string())?;
    
    let font_index = doc.add_font(&font);

    // 5. Render to Page
    let layout_root = template.render(&serde_json::json!({}));
    
    // Since we're rendering to a multi-page document, we can use the same 
    // `render_flow` algorithm from the N-API module, or manually iterate.
    // Let's use the Document abstraction from N-API style if available, or just render manually.
    
    // The Rust N-API backend actually exposes `render_flow`. Let's just use the direct layout
    // We can simulate the `render_flow` logic for testing pure core!
    
    let width = 595.0;
    let height = 842.0;
    let margin_top = 50.0;
    let margin_bottom = 50.0;
    let margin_left = 40.0;
    let margin_right = 40.0;
    
    let content_width = width - margin_left - margin_right;
    let body_available_height = height - margin_top - margin_bottom;
    
    let mut current_node = Some(layout_root.clone());
    let mut current_page = 1;
    
    while let Some(node) = current_node {
        let mut page = ai_pdf_writer::core::page::Page::new(width, height);
        let context = ai_pdf_writer::core::layout::PageContext { current: current_page, total: 2 }; // Just mock total
        
        let body_area = ai_pdf_writer::core::layout::Rect {
            x: margin_left,
            y: height - margin_top, // top-down
            width: content_width,
            height: body_available_height,
        };
        
        match node.split(content_width, body_available_height, &font) {
            ai_pdf_writer::core::layout::SplitAction::Fit | ai_pdf_writer::core::layout::SplitAction::Push => {
                node.render(&mut page, body_area, &font, font_index, &context);
                doc.add_page(&page)?;
                current_node = None;
            },
            ai_pdf_writer::core::layout::SplitAction::Split(head, tail) => {
                head.render(&mut page, body_area, &font, font_index, &context);
                doc.add_page(&page)?;
                current_node = Some(tail);
            }
        }
        current_page += 1;
    }

    // 6. Save the PDF
    doc.write_to("examples/comprehensive_out.pdf")?;
    println!("✅ PDF generated at examples/comprehensive_out.pdf");

    Ok(())
}

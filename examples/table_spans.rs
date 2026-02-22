use ai_pdf_writer::{
    core::{
        table::{Table, TableColumn, TableCell, TextAlign},
        page::Page,
        font::Font as CoreFont,
        document::Document,
    },
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut doc = Document::new();
    let font_bytes = include_bytes!("browser/assets/Roboto-Regular.ttf");
    let font = CoreFont::from_bytes(font_bytes.to_vec(), "Roboto-Regular".to_string())?;
    let font_index = doc.add_font(&font);
    
    let mut page = Page::new(595.0, 842.0); // A4
    page.text_with_font("Span Tests".to_string(), 50.0, 800.0, 24.0, font_index, &font);

    let mut table = Table::new(vec![
        TableColumn { header: "C1".into(), width: 100.0, align: TextAlign::Left, field: None },
        TableColumn { header: "C2".into(), width: 100.0, align: TextAlign::Left, field: None },
        TableColumn { header: "C3".into(), width: 100.0, align: TextAlign::Left, field: None },
        TableColumn { header: "C4".into(), width: 100.0, align: TextAlign::Left, field: None },
    ]);

    table.add_row(vec![
        TableCell { content: "R1C1 (Span 2)".into(), colspan: 2, rowspan: 1 },
        TableCell { content: "R1C3".into(), colspan: 1, rowspan: 1 },
        TableCell { content: "R1C4".into(), colspan: 1, rowspan: 1 },
    ]);

    table.add_row(vec![
        TableCell { content: "R2C1 (RowSpan 2)".into(), colspan: 1, rowspan: 2 },
        TableCell { content: "R2C2".into(), colspan: 1, rowspan: 1 },
        TableCell { content: "R2C3".into(), colspan: 1, rowspan: 1 },
        TableCell { content: "R2C4".into(), colspan: 1, rowspan: 1 },
    ]);

    table.add_row(vec![
        TableCell { content: "R3C2".into(), colspan: 1, rowspan: 1 },
        TableCell { content: "R3C3 (Span 2)".into(), colspan: 2, rowspan: 1 },
    ]);

    table.settings.striped = true;

    page.draw_table(&table, 50.0, 750.0, &font, font_index);
    doc.add_page(&page)?;

    doc.write_to("examples/test_spans.pdf")?;
    println!("Generated examples/test_spans.pdf");
    
    Ok(())
}

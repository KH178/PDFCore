use std::collections::{HashMap, HashSet};
use crate::core::font::Font;
use crate::core::writer::escape_string;
use crate::core::table::{Table, TextAlign};

/// Represents a single page in a PDF document
#[derive(Debug, Clone)]
pub struct Page {
    pub width: f32,
    pub height: f32,
    pub content: Vec<u8>,
    pub used_glyphs: HashMap<usize, HashSet<u16>>,  // font_index -> glyph_ids
    pub used_images: HashSet<u32>, // image_index
}

impl Page {
    /// Create a new page with specified dimensions
    pub fn new(width: f64, height: f64) -> Self {
        Page {
            width: width as f32,
            height: height as f32,
            content: Vec::new(),
            used_glyphs: HashMap::new(),
            used_images: HashSet::new(),
        }
    }
    
    /// Add text to the page at specified position with given font size
    pub fn text(&mut self, text: String, x: f64, y: f64, size: f64) -> &mut Self {
        let content = format!("BT /F1 {} Tf {} {} Td ({}) Tj ET ", size, x, y, escape_string(&text));
        self.content.extend(content.into_bytes());
        self
    }
    
    /// Add text to the page using a custom font (font_index + 2 for /F2, /F3, etc.)
    /// /F1 is reserved for built-in Helvetica
    /// Requires font reference to track glyph usage for subsetting
    pub fn text_with_font(&mut self, text: String, x: f64, y: f64, size: f64, font_index: u32, font: &Font) -> &mut Self {
        // Shape text to get glyph IDs
        let shaped = font.shape_text(&text, size);
        
        // Track used glyphs for subsetting
        self.used_glyphs
            .entry(font_index as usize)
            .or_insert_with(HashSet::new)
            .extend(shaped.iter().map(|g| g.glyph_id));
        
        // Font names: /F1 = Helvetica (built-in), /F2 = first custom font, /F3 = second, etc.
        let font_name = format!("F{}", font_index + 2);
        let content = format!("BT /{} {} Tf {} {} Td ({}) Tj ET ", font_name, size, x, y, escape_string(&text));
        self.content.extend(content.into_bytes());
        self
    }

    /// Add multiline text with wrapping
    pub fn text_multiline(&mut self, text: String, x: f64, y: f64, width: f64, size: f64, font_index: u32, font: &Font) -> &mut Self {
        let words: Vec<&str> = text.split_whitespace().collect();
        let space_width = font.measure_text(" ", size);
        let leading = size * 1.2; // Default line height
        
        let mut current_x = x;
        let mut current_y = y;
        let mut buffer = String::new();
        let mut current_width = 0.0;
        
        for word in words {
            let word_width = font.measure_text(word, size);
            
            if current_width + word_width > width {
                // Draw current line
                if !buffer.is_empty() {
                     // We need to call self.text_with_font, but we can't easily recurse with &mut self in a loop if we are not careful.
                     // But here we are just calling a method on self. It works fine.
                     self.text_with_font(buffer.trim().to_string(), x, current_y, size, font_index, font);
                }
                
                // Reset for next line
                buffer.clear();
                buffer.push_str(word);
                buffer.push(' ');
                current_width = word_width + space_width;
                current_y -= leading; // Move DOWN (PDF coords)
            } else {
                buffer.push_str(word);
                buffer.push(' ');
                current_width += word_width + space_width;
            }
        }
        
        // Draw last line
        if !buffer.is_empty() {
             self.text_with_font(buffer.trim().to_string(), x, current_y, size, font_index, font);
        }
        
        self
    }

    /// Draw an image on the page
    /// image_index is the index returned by document.add_image()
    pub fn draw_image(&mut self, image_index: u32, x: f64, y: f64, width: f64, height: f64) -> &mut Self {
        self.used_images.insert(image_index);
        
        // Save graphics state, transform, draw image, restore graphics state
        // PDF image coordinate system is 1x1 at (0,0), so we translate and scale
        let content = format!(
            "q {} 0 0 {} {} {} cm /Im{} Do Q ",
            width, height, x, y, image_index
        );
        self.content.extend(content.into_bytes());
        self
    }

    /// Draw a line from (x1, y1) to (x2, y2)
    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, width: f64) -> &mut Self {
        let content = format!(
            "{} w {} {} m {} {} l S ",
            width, x1, y1, x2, y2
        );
        self.content.extend(content.into_bytes());
        self
    }
    
    /// Draw a rectangle (stroke)
    pub fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, width: f64) -> &mut Self {
        let content = format!(
            "{} w {} {} {} {} re S ",
            width, x, y, w, h
        );
        self.content.extend(content.into_bytes());
        self
    }
    
    /// Draw a filled rectangle (gray)
    pub fn draw_fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, gray: f64) -> &mut Self {
        let content = format!(
            "{} g {} {} {} {} re f ",
            gray, x, y, w, h
        );
        self.content.extend(content.into_bytes());
        self
    }

    /// Draw a table starting at (x, y)
    /// Returns the y position after the table
    pub fn draw_table(&mut self, table: &Table, x: f64, y: f64, font: &Font) -> f64 {
        let mut current_y = y;
        let s = &table.settings;
        
        // 1. Draw Header
        let header_height = s.header_height;
        let total_width: f64 = table.columns.iter().map(|c| c.width).sum();
        
        // Header background
        self.draw_fill_rect(x, current_y - header_height, total_width, header_height, 0.9);
        self.draw_rect(x, current_y - header_height, total_width, header_height, s.border_width);
        
        // Header Content
        let mut current_x = x;
        for col in &table.columns {
            // Draw text centered vertically in header
            let text_y = current_y - (header_height / 2.0) - 4.0; // aprox centering
            self.text(col.header.clone(), current_x + s.padding, text_y, 10.0); // Header font size 10
            
            // Vertical border
            self.draw_rect(current_x, current_y - header_height, col.width, header_height, s.border_width);
            current_x += col.width;
        }
        current_y -= header_height;
        
        // 2. Draw Rows
        for row in &table.rows {
            // Calculate row height based on text wrapping (simplified: fixed height for now or use multiline)
            // For v2.0 MVP, let's use fixed cell_height from settings, or simplified wrap
            // Just use settings.cell_height to ensure predictability for now. 
            // Better: Measure max lines.
            
            let row_height = s.cell_height; // TODO: Auto-height
            
            current_x = x;
            for (i, cell_text) in row.iter().enumerate() {
                let width = if i < table.columns.len() { table.columns[i].width } else { 100.0 };
                
                // Draw text
                // Simple clipping or just drawing
                self.text_multiline(
                    cell_text.clone(), 
                    current_x + s.padding, 
                    current_y - s.padding - 8.0, // Top padding
                    width - (2.0 * s.padding), 
                    10.0, // Font size
                    0, // Font index (default)
                    font
                );
                
                // Vertical border
                self.draw_rect(current_x, current_y - row_height, width, row_height, s.border_width);
                
                current_x += width;
            }
            
            // Bottom border of row
            current_y -= row_height;
        }
        
        current_y
    }
}

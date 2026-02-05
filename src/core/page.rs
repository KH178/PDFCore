use std::collections::{HashMap, HashSet};
use crate::core::font::Font;
use crate::core::writer::escape_string;

/// Represents a single page in a PDF document
#[derive(Debug, Clone)]
pub struct Page {
    pub width: f32,
    pub height: f32,
    pub content: Vec<u8>,
    pub used_glyphs: HashMap<usize, HashSet<u16>>,  // font_index -> glyph_ids
}

impl Page {
    /// Create a new page with specified dimensions
    pub fn new(width: f64, height: f64) -> Self {
        Page {
            width: width as f32,
            height: height as f32,
            content: Vec::new(),
            used_glyphs: HashMap::new(),
        }
    }
    
    /// Add text to the page at specified position with given font size
    pub fn text(&mut self, text: String, x: f64, y: f64, size: f64) -> &mut Self {
        let content = format!("BT /F1 {} Tf {} {} Td ({}) Tj ET", size, x, y, escape_string(&text));
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
        let content = format!("BT /{} {} Tf {} {} Td ({}) Tj ET", font_name, size, x, y, escape_string(&text));
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
}

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
        
        // Convert glyph IDs to Hex string (Big Endian)
        let mut hex_content = String::new();
        hex_content.push('<');
        for g in &shaped {
            // Write u16 as 4 hex digits
            hex_content.push_str(&format!("{:04x}", g.glyph_id));
        }
        hex_content.push('>');
        
        // Ensure black color (0 g) and text object
        let content = format!("q 0 g BT /{} {} Tf {} {} Td {} Tj ET Q ", font_name, size, x, y, hex_content);
        self.content.extend(content.into_bytes());
        self
    }

    
    /// Calculate how many lines are needed for text with wrapping
    fn calculate_text_lines(text: &str, width: f64, size: f64, font: &Font) -> usize {
        if text.is_empty() {
            return 1;
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut buffer = Vec::new();
        let mut line_count = 0;
        
        for word in words {
            // Check if word alone is wider than available width
            let word_width = font.measure_text(word, size);
            
            if word_width > width {
                // Word needs character-level breaking
                // First, count the current buffer as a line if not empty
                if !buffer.is_empty() {
                    line_count += 1;
                    buffer.clear();
                }
                
                // Count lines needed for this word broken at character level
                let chars: Vec<char> = word.chars().collect();
                let mut char_buffer = String::new();
                
                for ch in chars {
                    let test_str = format!("{}{}", char_buffer, ch);
                    let test_width = font.measure_text(&test_str, size);
                    
                    if test_width <= width {
                        char_buffer.push(ch);
                    } else {
                        if !char_buffer.is_empty() {
                            line_count += 1;
                        }
                        char_buffer.clear();
                        char_buffer.push(ch);
                    }
                }
                
                // Count the last character buffer line
                if !char_buffer.is_empty() {
                    line_count += 1;
                }
            } else {
                // Try adding this word to the buffer
                let mut test_line = buffer.clone();
                test_line.push(word);
                let test_text = test_line.join(" ");
                let test_width = font.measure_text(&test_text, size);
                
                if test_width <= width {
                    // Word fits, add it to buffer
                    buffer.push(word);
                } else {
                    // Word doesn't fit
                    if !buffer.is_empty() {
                        // Complete the current line
                        line_count += 1;
                        buffer.clear();
                    }
                    // Start new line with this word
                    buffer.push(word);
                }
            }
        }
        
        // Count the last line
        if !buffer.is_empty() {
            line_count += 1;
        }
        
        line_count.max(1) // At least 1 line
    }
    
    /// Add multiline text with wrapping
    pub fn text_multiline(&mut self, text: String, x: f64, y: f64, width: f64, size: f64, font_index: u32, font: &Font) -> &mut Self {
        let words: Vec<&str> = text.split_whitespace().collect();
        let leading = size * 1.2; // Default line height
        
        let mut current_y = y;
        let mut buffer = Vec::new();
        
        for word in words {
            // Check if word alone is wider than available width
            let word_width = font.measure_text(word, size);
            
            if word_width > width {
                // Word is too long - need to break it at character level
                // First, flush current buffer
                if !buffer.is_empty() {
                    let line_text = buffer.join(" ");
                    self.text_with_font(line_text, x, current_y, size, font_index, font);
                    current_y -= leading;
                    buffer.clear();
                }
                
                // Break the word character by character
                let chars: Vec<char> = word.chars().collect();
                let mut char_buffer = String::new();
                
                for ch in chars {
                    let test_str = format!("{}{}", char_buffer, ch);
                    let test_width = font.measure_text(&test_str, size);
                    
                    if test_width <= width {
                        char_buffer.push(ch);
                    } else {
                        // Render current char_buffer and start new line
                        if !char_buffer.is_empty() {
                            self.text_with_font(char_buffer.clone(), x, current_y, size, font_index, font);
                            current_y -= leading;
                        }
                        char_buffer.clear();
                        char_buffer.push(ch);
                    }
                }
                
                // Render remaining characters
                if !char_buffer.is_empty() {
                    self.text_with_font(char_buffer, x, current_y, size, font_index, font);
                    current_y -= leading;
                }
            } else {
                // Try adding this word to the buffer
                let mut test_line = buffer.clone();
                test_line.push(word);
                let test_text = test_line.join(" ");
                let test_width = font.measure_text(&test_text, size);
                
                if test_width <= width {
                    // Word fits, add it to buffer
                    buffer.push(word);
                } else {
                    // Buffer with this word doesn't fit
                    if !buffer.is_empty() {
                        // Draw current buffer first
                        let line_text = buffer.join(" ");
                        self.text_with_font(line_text, x, current_y, size, font_index, font);
                        current_y -= leading;
                        buffer.clear();
                    }
                    
                    // Add word to new line
                    buffer.push(word);
                }
            }
        }
        
        // Draw last line
        if !buffer.is_empty() {
            let line_text = buffer.join(" ");
            self.text_with_font(line_text, x, current_y, size, font_index, font);
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
    /// Draw a table with specific font index
    pub fn draw_table(&mut self, table: &Table, x: f64, y: f64, font: &Font, font_index: u32) -> f64 {
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
            // Header always uses same font as body? Or maybe bold?
            // For now use same font
            self.text_with_font(col.header.clone(), current_x + s.padding, text_y, 10.0, font_index, font);
            
            // Vertical border
            self.draw_rect(current_x, current_y - header_height, col.width, header_height, s.border_width);
            current_x += col.width;
        }
        current_y -= header_height;
        
        // 2. Draw Rows
        for row in &table.rows {
            // Calculate required row height based on content
            let font_size = s.font_size; // Use font size from settings
            let leading = font_size * 1.2;
            let mut max_lines = 1;
            
            // Check all cells in this row to find the maximum number of lines needed
            for (i, cell_text) in row.iter().enumerate() {
                let col_width = if i < table.columns.len() { table.columns[i].width } else { 100.0 };
                let available_width = col_width - (2.0 * s.padding);
                let lines = Page::calculate_text_lines(cell_text, available_width, font_size, font);
                max_lines = max_lines.max(lines);
            }
            
            // Calculate row height: (lines * leading) + padding + extra space
            let content_height = max_lines as f64 * leading;
            let row_height = content_height + (2.0 * s.padding) + 8.0;
            
            current_x = x;
            for (i, cell_text) in row.iter().enumerate() {
                let width = if i < table.columns.len() { table.columns[i].width } else { 100.0 };
                
                // Draw text
                self.text_multiline(
                    cell_text.clone(), 
                    current_x + s.padding, 
                    current_y - s.padding - 8.0, // Top padding
                    width - (2.0 * s.padding), 
                    font_size,
                    font_index, // Use passed font index
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

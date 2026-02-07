use crate::core::page::Page;
use crate::core::font::Font;
use crate::core::text;
use crate::core::table::Table;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct PageContext {
    pub current: usize,
    pub total: usize,
}

impl Default for PageContext {
    fn default() -> Self {
        PageContext { current: 1, total: 1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Constraints {
    pub min_width: f64,
    pub max_width: f64,
    pub min_height: f64,
    pub max_height: f64,
}

impl Constraints {
    pub fn loose(width: f64, height: f64) -> Self {
        Constraints {
            min_width: 0.0,
            max_width: width,
            min_height: 0.0,
            max_height: height,
        }
    }
}

// Return type for split operation
pub enum SplitResult {
    None, // Used up all content, nothing left
    Split(Arc<dyn LayoutNode>, Option<Arc<dyn LayoutNode>>), // (Head, Tail). Tail is Option because sometimes we split but consume all? No.
    // If we split, we MUST have a Tail, unless implied?
    // Let's match plan: Split(Head, Tail)
    // Actually, plan said Split(Head, Tail).
    // Let's use: Split(Arc<dyn LayoutNode>, Arc<dyn LayoutNode>)
    // Wait, if Text splits into "Hello" and "World", we have two nodes.
    // If Text fits "Hello World", we return None (it fit).
    // If Text doesn't fit at all, we return Push?
    // Plan: NoteNone, Split(Head, Tail), Push.
}

// #[derive(Debug)]
pub enum SplitAction {
    Fit, // Fits completely
    Split(Arc<dyn LayoutNode>, Arc<dyn LayoutNode>), // (Head, Tail)
    Push, // Does not fit at all (or too small to split meaningfully)
}

/// A node in the layout tree that can size, position, and render itself.
pub trait LayoutNode {
    /// Calculate the size this node wants to be, given the constraints.
    fn measure(&self, constraints: Constraints, font: &Font) -> Size;
    
    /// Attempt to split this node to fit in available height (and width for wrapping context)
    fn split(&self, available_width: f64, available_height: f64, font: &Font) -> SplitAction;

    /// Draw the node onto the page within the given area.
    fn render(&self, page: &mut Page, area: Rect, font: &Font, font_index: u32, context: &PageContext);
}

// --- Components ---

pub struct Column {
    pub children: Vec<Arc<dyn LayoutNode>>,
    pub spacing: f64,
}

impl LayoutNode for Column {
    fn measure(&self, constraints: Constraints, font: &Font) -> Size {
        let mut width: f64 = 0.0;
        let mut height: f64 = 0.0;
        
        for child in &self.children {
            let child_size = child.measure(constraints, font);
            width = width.max(child_size.width);
            height += child_size.height + self.spacing;
        }
        
        // Remove last spacing if children exist
        if !self.children.is_empty() {
            height -= self.spacing;
        }
        
        Size { width: width.max(constraints.min_width), height }
    }

    fn render(&self, page: &mut Page, area: Rect, font: &Font, font_index: u32, context: &PageContext) {
        let mut y = area.y;
        
        for child in &self.children {
            let size = child.measure(Constraints::loose(area.width, f64::INFINITY), font);
            let child_area = Rect {
                x: area.x,
                y,
                width: area.width, 
                height: size.height,
            };
            child.render(page, child_area, font, font_index, context);
            y -= size.height + self.spacing; 
        }
    }

    fn split(&self, available_width: f64, available_height: f64, font: &Font) -> SplitAction {
        let mut used_height = 0.0;
        let mut split_index = None;
        let mut split_node_parts = None; // (Head, Tail) if a node splits

        for (i, child) in self.children.iter().enumerate() {
            // Measure child
            // Note: Column passes its full width as constraint generally.
            // But here we use available_width passed from parent.
            let constraints = Constraints::loose(available_width, f64::INFINITY);
            let size = child.measure(constraints, font);
            
            // Check if adding this child (plus spacing) exceeds available
            let spacing = if i > 0 { self.spacing } else { 0.0 };
            
            // Use a small safety margin to prevent items from sticking to the very bottom edge and potentially being clipped by PDF viewers or rounding errors.
            let safety_margin = 5.0;

            if used_height + spacing + size.height > available_height - safety_margin {
                // Overflow!
                // Can we split this child?
                let remaining_height = available_height - (used_height + spacing);
                
                // If remaining_height is tiny (e.g. < 0), we must push
                if remaining_height <= 0.0 {
                    split_index = Some(i);
                    // Child i is pushed entirely
                    break;
                }

                match child.split(available_width, remaining_height, font) {
                    SplitAction::Fit => {
                         used_height += spacing + size.height;
                    },
                    SplitAction::Push => {
                        // Child cannot fit in remaining.
                        split_index = Some(i);
                        break;
                    },
                    SplitAction::Split(head, tail) => {
                        // Child split. 
                        // Head goes to this column. Tail goes to next column.
                        split_index = Some(i);
                        split_node_parts = Some((head, tail));
                        break;
                    }
                }
            } else {
                used_height += spacing + size.height;
            }
        }

        if let Some(idx) = split_index {
            // Create Head Column (children 0..idx, plus potential head part)
            let mut head_children = self.children[0..idx].to_vec();
            
            // Create Tail Column (potential tail part, plus children idx+1..end)
            let mut tail_children = Vec::new();
            
            if let Some((head_part, tail_part)) = split_node_parts {
                head_children.push(head_part);
                tail_children.push(tail_part);
                // Add remaining existing children
                if idx + 1 < self.children.len() {
                    tail_children.extend_from_slice(&self.children[idx+1..]);
                }
            } else {
                // No split parts, meaning child[idx] was Pushed entirely to tail
                tail_children.extend_from_slice(&self.children[idx..]);
            }
            
            // Return split
            // If head_children empty, we pushed everything? Then we return Push.
            if head_children.is_empty() {
                return SplitAction::Push;
            }
            
            let head_col: Arc<dyn LayoutNode> = Arc::new(Column { children: head_children, spacing: self.spacing });
            let tail_col: Arc<dyn LayoutNode> = Arc::new(Column { children: tail_children, spacing: self.spacing });
            
            SplitAction::Split(head_col, tail_col)
        } else {
            // Loop finished, everything fits
            SplitAction::Fit
        }
    }
}

pub struct Row {
    pub children: Vec<Arc<dyn LayoutNode>>,
    pub spacing: f64,
}

impl LayoutNode for Row {
    fn measure(&self, constraints: Constraints, font: &Font) -> Size {
        let mut width: f64 = 0.0;
        let mut height: f64 = 0.0;
        
        for child in &self.children {
            let child_size = child.measure(constraints, font);
            width += child_size.width + self.spacing;
            height = height.max(child_size.height);
        }

        if !self.children.is_empty() {
             width -= self.spacing;
        }

        Size { width, height }
    }

    fn render(&self, page: &mut Page, area: Rect, font: &Font, font_index: u32, context: &PageContext) {
        let mut x = area.x;
        
        for child in &self.children {
             let size = child.measure(Constraints::loose(f64::INFINITY, area.height), font);
             let child_area = Rect {
                x,
                y: area.y,
                width: size.width, 
                height: area.height, 
            };
            child.render(page, child_area, font, font_index, context);
            x += size.width + self.spacing;
        }
    }

    fn split(&self, _available_width: f64, available_height: f64, font: &Font) -> SplitAction {
        let size = self.measure(Constraints::loose(f64::INFINITY, f64::INFINITY), font);
        if size.height <= available_height {
            SplitAction::Fit
        } else {
            SplitAction::Push
        }
    }
}

pub struct TextNode {
    pub text: String,
    pub size: f64,
    pub color: Option<crate::core::color::Color>,
    pub background_color: Option<crate::core::color::Color>,
}

impl LayoutNode for TextNode {
    fn measure(&self, constraints: Constraints, font: &Font) -> Size {
        // Compute raw width of text (unwrapped)
        let raw_width = font.measure_text(&self.text, self.size);
        
        // Determine actual width to use
        let width = if constraints.max_width.is_finite() {
            raw_width.min(constraints.max_width)
        } else {
            raw_width
        };
        
        let lines = text::calculate_text_lines(&self.text, width, self.size, font);
        let leading = self.size * 1.2;
        
        Size { width, height: lines as f64 * leading }
    }

    fn render(&self, page: &mut Page, area: Rect, font: &Font, font_index: u32, _context: &PageContext) {
        // Draw background first if specified
        // area.y is TOP of text area, but PDF rectangles use bottom-left coordinates
        if let Some(bg_color) = self.background_color {
            let bottom_y = area.y - area.height;
            page.draw_rect_filled(area.x, bottom_y, area.width, area.height, bg_color);
        }
        
        // Draw text with color on top of background
        let color = self.color.unwrap_or(crate::core::color::Color::black());
        page.text_multiline_colored(self.text.clone(), area.x, area.y, area.width, self.size, font_index, font, color);
    }

    fn split(&self, available_width: f64, available_height: f64, font: &Font) -> SplitAction {
        let leading = self.size * 1.2;
        let max_lines = (available_height / leading).floor() as usize;
        
        // If we can't fit even one line, Push
        if max_lines == 0 {
            return SplitAction::Push;
        }

        // Use helper to split
        // text::split_text_at_lines will measure and return (Head, Tail)
        let (head, tail_opt) = text::split_text_at_lines(&self.text, available_width, self.size, font, max_lines);
        
        if let Some(tail) = tail_opt {
            let head_node: Arc<dyn LayoutNode> = Arc::new(TextNode { text: head, size: self.size, color: self.color, background_color: self.background_color });
            let tail_node: Arc<dyn LayoutNode> = Arc::new(TextNode { text: tail, size: self.size, color: self.color, background_color: self.background_color });
            SplitAction::Split(head_node, tail_node)
        } else {
            // Fits completely
            SplitAction::Fit
        }
    }
}

pub struct Container {
    pub child: Arc<dyn LayoutNode>,
    pub padding: f64,
    pub border_width: f64,
}

impl LayoutNode for Container {
    fn measure(&self, constraints: Constraints, font: &Font) -> Size {
        // Decrease constraints by padding (2x) and border (2x)
        let reduction = (self.padding + self.border_width) * 2.0;
        
        let child_constraints = Constraints {
            min_width: (constraints.min_width - reduction).max(0.0),
            max_width: (constraints.max_width - reduction).max(0.0),
            min_height: (constraints.min_height - reduction).max(0.0),
            max_height: (constraints.max_height - reduction).max(0.0),
        };
        
        let child_size = self.child.measure(child_constraints, font);
        
        Size {
            width: child_size.width + reduction,
            height: child_size.height + reduction,
        }
    }

    fn render(&self, page: &mut Page, area: Rect, font: &Font, font_index: u32, context: &PageContext) {
        // Draw border if width > 0
        if self.border_width > 0.0 {
            // PDF rect is bottom-up. area.y is TOP.
            // We must draw from bottom-left: y - height
            let bottom_y = area.y - area.height;
            page.draw_rect(area.x, bottom_y, area.width, area.height, self.border_width);
        }
        
        let reduction = self.padding + self.border_width;
        let child_area = Rect {
            x: area.x + reduction,
            y: area.y - reduction, // PDF: Y goes UP, so "inner" top is Y - padding
            width: area.width - (reduction * 2.0),
            height: area.height - (reduction * 2.0),
        };
        
        self.child.render(page, child_area, font, font_index, context);
    }

    fn split(&self, available_width: f64, available_height: f64, font: &Font) -> SplitAction {
        let reduction = (self.padding + self.border_width) * 2.0;
        let child_avail_h = available_height - reduction;
        let child_avail_w = available_width - reduction;

        if child_avail_h <= 0.0 {
             return SplitAction::Push; 
        }

        match self.child.split(child_avail_w, child_avail_h, font) {
            SplitAction::Fit => SplitAction::Fit,
            SplitAction::Push => SplitAction::Push,
            SplitAction::Split(head, tail) => {
                // Wrap head and tail in new Containers with same padding/border
                let head_container: Arc<dyn LayoutNode> = Arc::new(Container { child: head, padding: self.padding, border_width: self.border_width });
                let tail_container: Arc<dyn LayoutNode> = Arc::new(Container { child: tail, padding: self.padding, border_width: self.border_width });
                SplitAction::Split(head_container, tail_container)
            }
        }
    }
}

pub struct ImageNode {
    pub image_index: u32,
    pub width: f64,
    pub height: f64,
}

impl LayoutNode for ImageNode {
    fn measure(&self, constraints: Constraints, _: &Font) -> Size {
        // Image has fixed intrinsic size, but respects constraints if smaller?
        // For MVP, return requested size confined by constraints.
        Size {
            width: self.width.min(constraints.max_width),
            height: self.height.min(constraints.max_height),
        }
    }

    fn render(&self, page: &mut Page, area: Rect, _: &Font, _font_index: u32, _context: &PageContext) {
        // Draw image fitting in the area. 
        // area.y is top. draw_image usually takes bottom-left?
        // Wait, page.draw_image(index, x, y, w, h). 
        // In core/page.rs, draw_image draws at x,y with w,h.
        // If Y is bottom-left, we need to convert.
        // But our layout engine "Y" convention assumes top-down flow in `Column`.
        // `Column` implementation: `y -= size.height`.
        // `Page.render_layout`: `y` is passed as top anchor.
        // So `area.y` passed to render is the TOP of the element.
        // If `draw_image` expects bottom-left, we must compute:
        // bottom_y = area.y - area.height.
        
        // Let's check Page::draw_image implementation in core/page.rs.
        // Step 1169: "cm" operator. `x y width height re`. Usually PDF uses bottom-left.
        // If `draw_image` uses `x y width height` directly in `cm`, it positions the image's bottom-left at (x, y).
        
        // So:
        let bottom_y = area.y - area.height;
        page.draw_image(self.image_index, area.x, bottom_y, area.width, area.height);
    }

    fn split(&self, _available_width: f64, available_height: f64, _: &Font) -> SplitAction {
        if self.height <= available_height {
            SplitAction::Fit
        } else {
            SplitAction::Push
        }
    }
}

// TableNode implementation
#[derive(Debug, Clone)]
pub struct TableNode {
    pub table: Table,
}

impl LayoutNode for TableNode {
    fn measure(&self, _constraints: Constraints, font: &Font) -> Size {
        // Table width is determined by columns (fixed)
        let width: f64 = self.table.columns.iter().map(|c| c.width).sum();
        
        let s = &self.table.settings;
        let mut height = s.header_height;
        let font_size = s.font_size;
        let leading = font_size * 1.2;

        for row in &self.table.rows {
             let mut max_lines = 1;
             for (i, cell_text) in row.iter().enumerate() {
                let col_width = if i < self.table.columns.len() { self.table.columns[i].width } else { 100.0 };
                let available_width = col_width - (2.0 * s.padding);
                let lines = text::calculate_text_lines(cell_text, available_width, font_size, font);
                max_lines = max_lines.max(lines);
             }
             let content_height = max_lines as f64 * leading;
             let row_height = content_height + (2.0 * s.padding) + 8.0;
             height += row_height;
        }

        Size { width, height }
    }

    fn render(&self, page: &mut Page, area: Rect, font: &Font, font_index: u32, _context: &PageContext) {
        page.draw_table(&self.table, area.x, area.y, font, font_index);
    }

    fn split(&self, _available_width: f64, available_height: f64, font: &Font) -> SplitAction {
         let s = &self.table.settings;
         let header_height = s.header_height;
         
         // If we allow table to split, head requires header_height.
         // Remaining for data = available_height - header_height.
         let data_available = available_height - header_height;
         
         if data_available <= 0.0 {
             return SplitAction::Push; 
         }

         let font_size = s.font_size;
         let leading = font_size * 1.2;
         
         let mut current_height = 0.0;
         let mut split_index = None;
         
         for (i, row) in self.table.rows.iter().enumerate() {
             // Calculate row height
             let mut max_lines = 1;
             for (j, cell_text) in row.iter().enumerate() {
                let col_width = if j < self.table.columns.len() { self.table.columns[j].width } else { 100.0 };
                let available_width = col_width - (2.0 * s.padding);
                let lines = text::calculate_text_lines(cell_text, available_width, font_size, font);
                max_lines = max_lines.max(lines);
             }
             let content_height = max_lines as f64 * leading;
             let row_height = content_height + (2.0 * s.padding) + 8.0;
             
             if current_height + row_height > data_available {
                 // Split here. This row (i) does not fit.
                 // So Head is 0..i. Tail is i..end.
                 // If i == 0, then NO rows fit. We must PUSH.
                 if i == 0 {
                     return SplitAction::Push;
                 }
                 split_index = Some(i);
                 break;
             }
             current_height += row_height;
         }
         
         if let Some(idx) = split_index {
             // Split
             let head_rows = self.table.rows[0..idx].to_vec();
             let tail_rows = self.table.rows[idx..].to_vec();
             
             let mut head_table = self.table.clone();
             head_table.rows = head_rows;
             
             let mut tail_table = self.table.clone();
             tail_table.rows = tail_rows;
             
             let head_node = Arc::new(TableNode { table: head_table });
             let tail_node = Arc::new(TableNode { table: tail_table });
             
             SplitAction::Split(head_node, tail_node)
         } else {
             SplitAction::Fit
         }
    }
}

// PageNumberNode: Renders text with {page} and {total} placeholders
#[derive(Debug, Clone)]
pub struct PageNumberNode {
    pub format: String,
    pub size: f64,
    pub align: String, // "left", "center", "right"
}

impl LayoutNode for PageNumberNode {
    fn measure(&self, constraints: Constraints, font: &Font) -> Size {
        // For measurement, replace placeholders with maximum expected values
        let sample_text = self.format.replace("{page}", "999").replace("{total}", "999");
        let lines = text::calculate_text_lines(&sample_text, constraints.max_width, self.size, font);
        let leading = self.size * 1.2;
        Size { width: constraints.max_width, height: lines as f64 * leading }
    }

    fn render(&self, page: &mut Page, area: Rect, font: &Font, font_index: u32, context: &PageContext) {
        // Replace placeholders with actual values from context
        let resolved_text = self.format
            .replace("{page}", &context.current.to_string())
            .replace("{total}", &context.total.to_string());
        
        let mut x = area.x;
        
        // Calculate position based on alignment
        if self.align == "right" {
            let text_width = font.measure_text(&resolved_text, self.size);
            x = area.x + area.width - text_width;
        } else if self.align == "center" {
            let text_width = font.measure_text(&resolved_text, self.size);
            x = area.x + (area.width - text_width) / 2.0;
        }
        
        page.text_multiline(resolved_text, x, area.y, area.width, self.size, font_index, font);
    }

    fn split(&self, _available_width: f64, available_height: f64, font: &Font) -> SplitAction {
        // Calculate height
        let lines = text::calculate_text_lines(&self.format, _available_width, self.size, font);
        let leading = self.size * 1.2;
        let height = lines as f64 * leading;
        
        if height <= available_height {
            SplitAction::Fit
        } else {
            SplitAction::Push
        }
    }
}


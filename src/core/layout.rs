use crate::core::page::Page;
use crate::core::font::Font;
use std::sync::Arc;

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

/// A node in the layout tree that can size, position, and render itself.
pub trait LayoutNode {
    /// Calculate the size this node wants to be, given the constraints.
    fn measure(&self, constraints: Constraints, font: &Font) -> Size;
    
    /// Assign a specific area to the node. 
    /// This is where cached positions would be stored if we had state.
    /// For this stateless pass, we might just use the result in render, 
    /// but usually layout engines store the computed rect.
    /// For simplicity in v3.0, render will take the Rect.
    fn layout(&mut self, area: Rect) {
        // Default impl (noop)
    }

    /// Draw the node onto the page within the given area.
    fn render(&self, page: &mut Page, area: Rect, font: &Font);
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

    fn render(&self, page: &mut Page, area: Rect, font: &Font) {
        let mut y = area.y;
        
        for child in &self.children {
            let size = child.measure(Constraints::loose(area.width, f64::INFINITY), font);
            let child_area = Rect {
                x: area.x,
                y,
                width: area.width, 
                height: size.height,
            };
            child.render(page, child_area, font);
            y -= size.height + self.spacing; 
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

    fn render(&self, page: &mut Page, area: Rect, font: &Font) {
        let mut x = area.x;
        
        for child in &self.children {
             let size = child.measure(Constraints::loose(f64::INFINITY, area.height), font);
             let child_area = Rect {
                x,
                y: area.y,
                width: size.width, 
                height: area.height, 
            };
            child.render(page, child_area, font);
            x += size.width + self.spacing;
        }
    }
}

pub struct TextNode {
    pub text: String,
    pub size: f64,
}

impl LayoutNode for TextNode {
    fn measure(&self, constraints: Constraints, font: &Font) -> Size {
        let width = constraints.max_width;
        // Calculate expected height based on wrapping
        // This duplicates logic in text_multiline a bit, but necessary.
        // For v3.0 MVP, let's use a helper or estimate.
        // Actually, let's just make text_multiline return the height used?
        // Or duplicate the split logic.
        
        let words: Vec<&str> = self.text.split_whitespace().collect();
        let space_width = font.measure_text(" ", self.size);
        let leading = self.size * 1.2;
        
        let mut current_width = 0.0;
        let mut lines = 1;
        
        for word in words {
            let word_width = font.measure_text(word, self.size);
            if current_width + word_width > width {
                lines += 1;
                current_width = word_width + space_width;
            } else {
                current_width += word_width + space_width;
            }
        }
        
        Size { width, height: lines as f64 * leading }
    }

    fn render(&self, page: &mut Page, area: Rect, font: &Font) {
        page.text_multiline(self.text.clone(), area.x, area.y, area.width, self.size, 0, font);
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

    fn render(&self, page: &mut Page, area: Rect, font: &Font) {
        // Draw border if width > 0
        if self.border_width > 0.0 {
            page.draw_rect(area.x, area.y, area.width, area.height, self.border_width);
        }
        
        let reduction = self.padding + self.border_width;
        let child_area = Rect {
            x: area.x + reduction,
            y: area.y - reduction, // PDF: Y goes UP, so "inner" top is Y - padding
            width: area.width - (reduction * 2.0),
            height: area.height - (reduction * 2.0),
        };
        
        self.child.render(page, child_area, font);
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

    fn render(&self, page: &mut Page, area: Rect, _: &Font) {
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
}


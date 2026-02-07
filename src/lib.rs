use napi_derive::napi;
use napi::{Result, Error, Status};
use std::io;

mod core;

use crate::core::font::Font as CoreFont;
use crate::core::page::Page as CorePage;
use crate::core::document::Document as CoreDocument;
use crate::core::image::Image as CoreImage;
use crate::core::table::{Table as CoreTable, TableColumn as CoreTableColumn, TextAlign as CoreTextAlign};
use crate::core::layout::{LayoutNode as CoreLayoutNode, Column as CoreColumn, Row as CoreRow, TextNode as CoreTextNode, Container as CoreContainer, ImageNode as CoreImageNode, Rect as CoreRect, Constraints as CoreConstraints, SplitAction, PageContext as CorePageContext};
use crate::core::template::Template as CoreTemplate;

// Helper to map IO errors to N-API errors
fn map_io_err(e: io::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// Represents a loaded font with parsing and shaping capabilities
#[napi]
pub struct Font {
    pub(crate) inner: CoreFont,
}

#[napi]
impl Font {
    /// Load a font from a file path
    #[napi(factory)]
    pub fn from_file(path: String, name: String) -> Result<Self> {
        let inner = CoreFont::from_file(&path, name).map_err(map_io_err)?;
        Ok(Font { inner })
    }
    
    /// Load a font from bytes (e.g., embedded font data)
    #[napi(factory)]
    pub fn from_bytes(data: Vec<u8>, name: String) -> Result<Self> {
        let inner = CoreFont::from_bytes(data, name).map_err(map_io_err)?;
        Ok(Font { inner })
    }
    
    /// Measure text width using shaping
    #[napi]
    pub fn measure_text(&self, text: String, size: f64) -> f64 {
        self.inner.measure_text(&text, size)
    }

    /// Shape text and return glyph IDs with positions
    #[napi]
    pub fn shape_text(&self, text: String, size: f64) -> Vec<ShapedGlyph> {
        self.inner.shape_text(&text, size)
            .into_iter()
            .map(|g| ShapedGlyph {
                glyph_id: g.glyph_id,
                x_advance: g.x_advance,
                y_advance: g.y_advance,
                x_offset: g.x_offset,
                y_offset: g.y_offset,
            })
            .collect()
    }
}

/// Represents a loaded image (JPEG or PNG)
#[napi]
pub struct Image {
    inner: CoreImage,
}

#[napi]
impl Image {
    /// Load an image from a file path
    #[napi(factory)]
    pub fn from_file(path: String) -> Result<Self> {
        let inner = CoreImage::from_file(&path).map_err(map_io_err)?;
        Ok(Image { inner })
    }

    /// Load an image from bytes
    #[napi(factory)]
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let inner = CoreImage::from_bytes(&data).map_err(map_io_err)?;
        Ok(Image { inner })
    }
}

/// Represents a shaped glyph with position and advance information
#[napi(object)]
pub struct ShapedGlyph {
    pub glyph_id: u16,
    pub x_advance: f64,
    pub y_advance: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

/// Column definition for Table
/// Color structure (RGB/RGBA)
#[napi(object)]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: Option<f64>,
}

/// Template for repeating headers and footers
#[napi(object)]
#[derive(Clone)]
pub struct PageTemplate {
    pub margin_top: Option<f64>,
    pub margin_bottom: Option<f64>,
}

#[napi(object)]
pub struct TableColumn {
    pub header: String,
    pub width: f64,
    pub align: Option<String>, // "Left", "Center", "Right"
    pub field: Option<String>,
}

/// Data Table with headers and rows
#[napi]
pub struct Table {
    inner: CoreTable,
}

#[napi]
impl Table {
    #[napi(constructor)]
    pub fn new(columns: Vec<TableColumn>) -> Self {
        let core_cols = columns.into_iter().map(|c| CoreTableColumn {
            header: c.header,
            width: c.width,
            align: match c.align.as_deref() {
                Some("Center") => CoreTextAlign::Center,
                Some("Right") => CoreTextAlign::Right,
                _ => CoreTextAlign::Left,
            },
            field: c.field,
        }).collect();
        
        Table {
            inner: CoreTable::new(core_cols),
        }
    }

    #[napi]
    pub fn add_row(&mut self, row: Vec<String>) {
        self.inner.add_row(row);
    }
    
    #[napi]
    pub fn set_font_size(&mut self, size: f64) {
        self.inner.settings.font_size = size;
    }
}



/// Template loaded from JSON
#[napi]
pub struct Template {
    inner: CoreTemplate,
}

#[napi]
impl Template {
    /// Load template from JSON string
    #[napi(factory)]
    pub fn from_json(json: String) -> Result<Self> {
        let inner = CoreTemplate::from_json(&json).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Template { inner })
    }

    /// Load template from .pdfCoret (zip) file
    #[napi(factory)]
    pub fn from_zip(path: String) -> Result<Self> {
        let inner = CoreTemplate::from_zip(&path).map_err(|e| Error::from_reason(e))?;
        Ok(Template { inner })
    }

    /// Convert template to a LayoutNode for rendering
    #[napi]
    pub fn to_layout(&self, data_json: Option<String>) -> Result<LayoutNode> {
        let data = if let Some(json) = data_json {
            serde_json::from_str(&json).map_err(|e| Error::from_reason(format!("Data JSON Error: {}", e)))?
        } else {
            serde_json::Value::Null
        };
        
        Ok(LayoutNode {
            inner: self.inner.render(&data)
        })
    }

    /// Alias for to_layout with data
    #[napi]
    pub fn render(&self, data_json: String) -> Result<LayoutNode> {
        self.to_layout(Some(data_json))
    }
}

use std::sync::Arc;

/// Opaque wrapper for a Layout Node
#[napi]
#[derive(Clone)]
pub struct LayoutNode {
     pub(crate) inner: Arc<dyn CoreLayoutNode>,
}

#[napi]
impl LayoutNode {
    /// Create a Column node
    #[napi(factory)]
    pub fn column(children: Vec<&LayoutNode>, spacing: Option<f64>) ->  Self {
        let core_children: Vec<Arc<dyn CoreLayoutNode>> = children.iter()
            .map(|n| n.inner.clone())
            .collect();
            
        let col = CoreColumn {
            children: core_children,
            spacing: spacing.unwrap_or(0.0),
        };
        
        LayoutNode { inner: Arc::new(col) }
    }
    
    #[napi(factory)]
    pub fn row(children: Vec<&LayoutNode>, spacing: Option<f64>) ->  Self {
        let core_children: Vec<Arc<dyn CoreLayoutNode>> = children.iter()
            .map(|n| n.inner.clone())
            .collect();
            
        let row = CoreRow {
            children: core_children,
            spacing: spacing.unwrap_or(0.0),
        };
        
        LayoutNode { inner: Arc::new(row) }
    }
    
    #[napi(factory)]
    pub fn text(text: String, size: f64, color: Option<Color>, background_color: Option<Color>) -> Self {
        let normalize = |c: Color| {
            if c.r > 1.0 || c.g > 1.0 || c.b > 1.0 {
                crate::core::color::Color::rgba(c.r / 255.0, c.g / 255.0, c.b / 255.0, c.a.unwrap_or(1.0))
            } else {
                crate::core::color::Color::rgba(c.r, c.g, c.b, c.a.unwrap_or(1.0))
            }
        };

        let core_color = color.map(normalize);
        let core_background_color = background_color.map(normalize);
        
        LayoutNode {
            inner: Arc::new(CoreTextNode { text, size, color: core_color, background_color: core_background_color }),
        }
    }
    
    #[napi(factory)]
    pub fn container(child: &LayoutNode, padding: Option<f64>, border: Option<f64>) -> Self {
        LayoutNode {
            inner: Arc::new(CoreContainer {
                child: child.inner.clone(),
                padding: padding.unwrap_or(0.0),
                border_width: border.unwrap_or(0.0),
            }),
        }
    }
    
    #[napi(factory)]
    pub fn image(image_index: u32, width: f64, height: f64) -> Self {
        LayoutNode { 
            inner: Arc::new(CoreImageNode {
                image_index,
                width,
                height,
            })
        }
    }

    #[napi(factory)]
    pub fn table(table: &Table) -> Self {
        LayoutNode {
            inner: Arc::new(crate::core::layout::TableNode {
                table: table.inner.clone(),
            }),
        }
    }

    #[napi(factory)]
    pub fn page_number(format: String, size: f64, align: Option<String>) -> Self {
        LayoutNode {
            inner: Arc::new(crate::core::layout::PageNumberNode {
                format,
                size,
                align: align.unwrap_or_else(|| "left".to_string()),
            }),
        }
    }
}

// ... ShapedGlyph ... 

/// Represents a single page in a PDF document
#[napi]
pub struct Page {
    inner: CorePage,
}

#[napi]
impl Page {
    /// Create a new page with specified dimensions
    #[napi(constructor)]
    pub fn new(width: f64, height: f64) -> Self {
        Page {
            inner: CorePage::new(width, height),
        }
    }

    /// Add text to the page using built-in font (Helvetica)
    #[napi]
    pub fn text(&mut self, text: String, x: f64, y: f64, size: f64) -> &Self {
        // CorePage needs &str, or String? core/page.rs text() takes String.
        self.inner.text(text, x, y, size);
        self
    }
    
    /// Add multiline text with wrapping
    #[napi]
    pub fn text_multiline(&mut self, text: String, x: f64, y: f64, width: f64, size: f64, font_index: u32, font: &Font) -> &Self {
        self.inner.text_multiline(text, x, y, width, size, font_index, &font.inner);
        self
    }
    
    /// Add text using a custom font (by index)
    #[napi]
    pub fn text_with_font(&mut self, text: String, x: f64, y: f64, size: f64, font_index: u32, font: &Font) -> &Self {
        self.inner.text_with_font(text, x, y, size, font_index, &font.inner);
        self
    }

    /// Draw a line
    #[napi]
    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, width: f64) -> &Self {
        self.inner.draw_line(x1, y1, x2, y2, width);
        self
    }

    /// Draw a rectangle (stroke)
    #[napi]
    pub fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, width: f64) -> &Self {
        self.inner.draw_rect(x, y, w, h, width);
        self
    }

    /// Draw a filled rectangle (gray)
    #[napi]
    pub fn draw_fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, gray: f64) -> &Self {
        self.inner.draw_fill_rect(x, y, w, h, gray);
        self
    }

    /// Draw a table
    #[napi]
    pub fn draw_table(&mut self, table: &Table, x: f64, y: f64, font: &Font, font_index: u32) -> f64 {
        self.inner.draw_table(&table.inner, x, y, &font.inner, font_index)
    }

    /// Draw an image
    #[napi]
    pub fn draw_image(&mut self, image_index: u32, x: f64, y: f64, width: f64, height: f64) -> &Self {
        // Core Page::draw_image takes image_index, x, y, width, height
        self.inner.draw_image(image_index, x, y, width, height);
        self
    }

    /// Render a declarative layout tree
    #[napi]
    pub fn render_layout(
        &mut self, 
        node: &LayoutNode, 
        x: f64, 
        y: f64, 
        width: f64, 
        font: &Font, 
        font_index: u32,
        current_page: Option<u32>,
        total_pages: Option<u32>
    ) {
        // Create constraints based on width (and infinite height for now?)
        // y in PDF is bottom-up, but layout engine might assume top-down relative to cached pos.
        // Our layout engine `render` takes a Rect.
        // We'll give it the bounding box.
        
        // Construct page context
        let context = if let (Some(c), Some(t)) = (current_page, total_pages) {
            crate::core::layout::PageContext {
                current: c as usize,
                total: t as usize,
            }
        } else {
            crate::core::layout::PageContext::default()
        };

        let constraints = CoreConstraints::loose(width, f64::INFINITY);
        let size = node.inner.measure(constraints, &font.inner);
        
        let area = CoreRect {
            x,
            y, // Note: In our current text_multiline, Y is the TOP baseline.
               // If layout engine assumes Y is top, it flows DOWN.
               // We need to ensure Y decreases. 
               // render() in layout components subtracts size.height.
               // So if we pass Y, it will draw from Y downwards.
            width: size.width,
            height: size.height,
        };
        
        node.inner.render(&mut self.inner, area, &font.inner, font_index, &context);
        
    }
}

/// Represents a PDF document with multiple pages
#[napi]
pub struct Document {
    inner: Option<CoreDocument>,
}

#[napi]
impl Document {
    /// Create a new empty document in buffered mode
    #[napi(constructor)]
    pub fn new() -> Self {
        Document {
            inner: Some(CoreDocument::new()),
        }
    }
    
    /// Create a new document in streaming mode
    /// Pages are written immediately as they're added
    #[napi(factory)]
    pub fn streaming(path: String) -> Result<Self> {
        let inner = CoreDocument::streaming(&path).map_err(map_io_err)?;
        Ok(Document { inner: Some(inner) })
    }
    
    /// Register a custom font with the document
    /// Returns the font index to use in page rendering
    #[napi]
    pub fn add_font(&mut self, font: &Font) -> Result<u32> {
        if let Some(doc) = &mut self.inner {
            Ok(doc.add_font(&font.inner))
        } else {
             Err(Error::new(Status::GenericFailure, "Document is finalized".to_string()))
        }
    }

    /// Register an image with the document
    /// Returns the image index to use in page rendering
    #[napi]
    pub fn add_image(&mut self, image: &Image) -> Result<u32> {
        if let Some(doc) = &mut self.inner {
            doc.add_image(&image.inner).map_err(map_io_err)
        } else {
             Err(Error::new(Status::GenericFailure, "Document is finalized".to_string()))
        }
    }
    
    /// Add a page to the document
    #[napi]
    pub fn add_page(&mut self, page: &Page) -> Result<()> {
        if let Some(doc) = &mut self.inner {
             doc.add_page(&page.inner).map_err(map_io_err)
        } else {
             Err(Error::new(Status::GenericFailure, "Document is finalized".to_string()))
        }
    }
    
    /// Finalize a streaming document
    #[napi]
    pub fn finalize(&mut self) -> Result<()> {
        if let Some(mut doc) = self.inner.take() {
            doc.finalize().map_err(map_io_err)
        } else {
            Err(Error::new(Status::GenericFailure, "Document is already finalized".to_string()))
        }
    }
    
    /// Write the document to a file (buffered mode)
    #[napi]
    pub fn write_to(&self, path: String) -> Result<()> {
        if let Some(doc) = &self.inner {
            doc.write_to(&path).map_err(map_io_err)
        } else {
            Err(Error::new(Status::GenericFailure, "Document is finalized".to_string()))
        }
    }

    /// Register assets from a loaded Template into this Document
    /// This is required if the template contains images.
    #[napi]
    pub fn register_template_assets(&mut self, template: &mut Template) -> Result<()> {
        if let Some(doc) = &mut self.inner {
             for (name, bytes) in &template.inner.assets {
                 let img = CoreImage::from_bytes(bytes).map_err(map_io_err)?;
                 let idx = doc.add_image(&img).map_err(map_io_err)?;
                 template.inner.asset_indices.insert(name.clone(), idx);
             }
             Ok(())
        } else {
             Err(Error::new(Status::GenericFailure, "Document is finalized".to_string()))
        }
    }

    /// Automatically paginate a layout tree across multiple pages
    #[napi]
    pub fn render_flow(
        &mut self, 
        node: &LayoutNode, 
        width: f64, 
        height: f64, 
        font: &Font, 
        font_index: u32,
        header: Option<&LayoutNode>,
        footer: Option<&LayoutNode>,
        template: Option<PageTemplate>
    ) -> Result<()> {
        let header_node = header.map(|h| h.inner.clone());
        let footer_node = footer.map(|f| f.inner.clone());
        let margin_top = template.as_ref().and_then(|t| t.margin_top).unwrap_or(0.0);
        let margin_bottom = template.as_ref().and_then(|t| t.margin_bottom).unwrap_or(0.0);

        // Pre-calculate fixed reserved space
        let constraints = CoreConstraints::loose(width, f64::INFINITY);
        let header_height = if let Some(h) = &header_node {
             h.measure(constraints, &font.inner).height
        } else { 0.0 };

        let footer_height = if let Some(f) = &footer_node {
             f.measure(constraints, &font.inner).height
        } else { 0.0 };
        
        let top_reserved = margin_top + header_height;
        let bottom_reserved = margin_bottom + footer_height;
        let body_available_height = height - top_reserved - bottom_reserved;
        let body_start_y = height - top_reserved;
        
        // Side margins for content (50pt left/right)
        let side_margin = 50.0;
        let content_width = width - (side_margin * 2.0);

        // === PASS 1: Dry Run - Count Total Pages ===
        let mut page_count = 0;
        let mut current_node = Some(node.inner.clone());
        
        while current_node.is_some() {
            page_count += 1;
            let node = current_node.unwrap();
            
            match node.split(content_width, body_available_height, &font.inner) {
                SplitAction::Fit | SplitAction::Push => {
                    current_node = None;
                },
                SplitAction::Split(_, tail) => {
                    current_node = Some(tail);
                }
            }
        }

        // === PASS 2: Real Render with Accurate Context ===
        let mut current_page = 1;
        let mut current_node = Some(node.inner.clone());

        while let Some(node) = current_node {
             let mut page = Page::new(width, height);
             
             let context = CorePageContext {
                 current: current_page,
                 total: page_count,
             };
             
             // 1. Render Header with context
             if let Some(h) = &header_node {
                 let header_area = CoreRect { x: side_margin, y: height - margin_top, width: content_width, height: header_height };
                 h.render(&mut page.inner, header_area, &font.inner, font_index, &context);
             }

             // 2. Render Footer at very bottom with context
             if let Some(f) = &footer_node {
                 let footer_y = margin_bottom;
                 let footer_area = CoreRect { x: side_margin, y: footer_y, width: content_width, height: footer_height };
                 f.render(&mut page.inner, footer_area, &font.inner, font_index, &context);
             }
             
             // 3. Render Body with side margins
             match node.split(content_width, body_available_height, &font.inner) {
                 SplitAction::Fit => {
                      page.render_layout(&LayoutNode { inner: node }, side_margin, body_start_y, content_width, font, font_index, Some(current_page as u32), Some(page_count as u32));
                      self.add_page(&page)?;
                      current_node = None;
                 },
                 SplitAction::Push => {
                      page.render_layout(&LayoutNode { inner: node }, side_margin, body_start_y, content_width, font, font_index, Some(current_page as u32), Some(page_count as u32));
                      self.add_page(&page)?;
                      current_node = None;
                 },
                 SplitAction::Split(head, tail) => {
                      page.render_layout(&LayoutNode { inner: head }, side_margin, body_start_y, content_width, font, font_index, Some(current_page as u32), Some(page_count as u32));
                      self.add_page(&page)?;
                      
                      current_node = Some(tail);
                 }
             }
             
             current_page += 1;
        }
        Ok(())
    }
}

// ... rest of file

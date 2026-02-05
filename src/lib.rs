use napi_derive::napi;
use napi::{Result, Error, Status};
use std::io;

mod core;

use crate::core::font::Font as CoreFont;
use crate::core::page::Page as CorePage;
use crate::core::document::Document as CoreDocument;
use crate::core::image::Image as CoreImage;
use crate::core::table::{Table as CoreTable, TableColumn as CoreTableColumn, TextAlign as CoreTextAlign};

// Helper to map IO errors to N-API errors
fn map_io_err(e: io::Error) -> Error {
    Error::from_reason(e.to_string())
}

/// Represents a loaded font with parsing and shaping capabilities
#[napi]
pub struct Font {
    inner: CoreFont,
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
}

/// Column definition for Table
#[napi(object)]
pub struct TableColumn {
    pub header: String,
    pub width: f64,
    pub align: Option<String>, // "Left", "Center", "Right"
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
        }).collect();
        
        Table {
            inner: CoreTable::new(core_cols),
        }
    }

    #[napi]
    pub fn add_row(&mut self, row: Vec<String>) {
        self.inner.add_row(row);
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
    
    /// Add text to the page at specified position with given font size
    #[napi]
    pub fn text(&mut self, text: String, x: f64, y: f64, size: f64) -> &Self {
        self.inner.text(text, x, y, size);
        self
    }
    
    /// Add text to the page using a custom font (font_index + 2 for /F2, /F3, etc.)
    /// /F1 is reserved for built-in Helvetica
    /// Requires font reference to track glyph usage for subsetting
    #[napi]
    pub fn text_with_font(&mut self, text: String, x: f64, y: f64, size: f64, font_index: u32, font: &Font) -> &Self {
        self.inner.text_with_font(text, x, y, size, font_index, &font.inner);
        self
    }

    /// Add multiline text with wrapping
    #[napi]
    pub fn text_multiline(&mut self, text: String, x: f64, y: f64, width: f64, size: f64, font_index: u32, font: &Font) -> &Self {
        self.inner.text_multiline(text, x, y, width, size, font_index, &font.inner);
        self
    }

    /// Draw an image on the page
    /// image_index is the index returned by document.addImage()
    #[napi]
    pub fn draw_image(&mut self, image_index: u32, x: f64, y: f64, width: f64, height: f64) -> &Self {
        self.inner.draw_image(image_index, x, y, width, height);
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
    pub fn draw_table(&mut self, table: &Table, x: f64, y: f64, font: &Font) -> f64 {
        self.inner.draw_table(&table.inner, x, y, &font.inner)
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
}

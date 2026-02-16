use wasm_bindgen::prelude::*;
use crate::core::font::Font as CoreFont;
use crate::core::page::Page as CorePage;
use crate::core::document::Document as CoreDocument;
use crate::core::image::Image as CoreImage;
use crate::core::template::Template as CoreTemplate;
use crate::core::layout::{LayoutNode as CoreLayoutNode};
use std::sync::Arc;

// WASM-specific error handling
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct WasmFont {
    inner: CoreFont,
}

#[wasm_bindgen]
impl WasmFont {
    #[wasm_bindgen]
    pub fn from_bytes(data: &[u8], name: String) -> Result<WasmFont, JsValue> {
        let inner = CoreFont::from_bytes(data.to_vec(), name)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmFont { inner })
    }
}

// Re-export core types wrapped in WASM-friendly structs

#[wasm_bindgen]
pub struct WasmTemplate {
    inner: CoreTemplate,
}

#[wasm_bindgen]
impl WasmTemplate {
    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Result<WasmTemplate, JsValue> {
        let inner = CoreTemplate::from_json(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmTemplate { inner })
    }
    
    // Zip loading in WASM requires bytes, not file path
    #[wasm_bindgen]
    pub fn from_zip_bytes(bytes: &[u8]) -> Result<WasmTemplate, JsValue> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        let inner = CoreTemplate::from_zip_reader(cursor)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(WasmTemplate { inner })
    }

    #[wasm_bindgen]
    pub fn render(&self, data_json: &str) -> Result<WasmLayoutNode, JsValue> {
        let data: serde_json::Value = serde_json::from_str(data_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid data JSON: {}", e)))?;
            
        Ok(WasmLayoutNode {
            inner: self.inner.render(&data)
        })
    }
}

#[wasm_bindgen]
pub struct WasmLayoutNode {
    inner: Arc<dyn CoreLayoutNode>,
}

#[wasm_bindgen]
pub struct WasmDocument {
    inner: CoreDocument,
}

#[wasm_bindgen]
impl WasmDocument {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmDocument {
        WasmDocument {
            inner: CoreDocument::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn add_page(&mut self, page: WasmPage) {
        self.inner.add_page(&page.inner);
    }

    #[wasm_bindgen]
    pub fn add_font(&mut self, font: &WasmFont) -> u32 {
        self.inner.add_font(&font.inner)
    }
    
    #[wasm_bindgen]
    pub fn save(&self) -> Result<Vec<u8>, JsValue> {
        let mut buffer = std::io::Cursor::new(Vec::new());
        // We need write_to_writer in core/document.rs
        self.inner.write_to_writer(&mut buffer)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(buffer.into_inner())
    }
}

#[wasm_bindgen]
pub struct WasmPage {
    inner: CorePage,
}

#[wasm_bindgen]
impl WasmPage {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f64, height: f64) -> WasmPage {
        WasmPage {
            inner: CorePage::new(width, height),
        }
    }
    
    #[wasm_bindgen]
    pub fn render_layout(&mut self, node: &WasmLayoutNode, font: &WasmFont, font_index: u32) {
        // Draw a test rectangle (FILLED BLACK) to verify rendering
        // unsafe { log("WASM: render_layout called"); }
        // self.inner.draw_fill_rect(50.0, 50.0, 200.0, 100.0, 0.0); // 0.0 = Black
        
        let area = crate::core::layout::Rect {
            x: 0.0,
            y: self.inner.height as f64, // PDF coordinates: 0,0 is bottom-left. But layout engine usually assumes top-left?
            // Wait, Layout Engine assumes top-left (y=0 is top).
            // But PDF coordinates are bottom-left.
            // Page::text args: x, y. 
            // Core::LayoutNode::render implementations use area.y as TOP.
            // And pass it to Page methods.
            // Let's check Page::text implementation. "BT ... x y Td". 
            // Native PDF is y=0 at bottom.
            // If we use top-left logic, y should be height - y_layout.
            // But CoreLayout seems to assume y down?
            // "y -= size.height" in Column. This implies y decreases.
            // So y starts at Top (e.g. 842) and goes down (to 0).
            // So area.y should be page.height.
            width: self.inner.width as f64,
            height: self.inner.height as f64,
        };
        
        let context = crate::core::layout::PageContext {
            current: 1,
            total: 1,
        };
        
        // Font index 0. In Document::add_page, custom fonts are F2, F3...
        // Built-in is F1.
        // If we pass a custom font, we need to register it with the document first?
        // Layout rendering just needs metrics. 
        // But render() puts "/F(index+2) Tf" instruction.
        // So we need to ensure the document KNOWS about this font index.
        // WasmPage doesn't know about Document!
        
        // IMPORTANT: WasmPage rendering adds content to the page buffer.
        // It uses "font_index" to refer to a font resource /Fn.
        // That resource must be defined in the Page Dictionary Properties when added to Document.
        // WasmDocument::add_page takes the page and embeds fonts.
        // CoreDocument::add_page uses `self.fonts`.
        
        // So we need to add the font to the DOCUMENT, get an ID, and pass that ID to render_layout?
        // OR: `render_layout` works on Page, but assumes font_index is valid.
        
        // For this MVP, let's assume we use:
        // 1. A custom font passed to render_layout (for metrics).
        // 2. We trigger "add_font" on the document later?
        // THIS IS TRICKY. The content stream refers to /Fn.
        // The Document must populate Resources with /Fn -> FontObject.
        // So Layout rendering and Document resource gathering must agree on index.
        
        // If we render layout on a detached Page, we don't know the index yet.
        // UNLESS we pass it.
        
        // Proposal:
        // 1. WasmDocument.add_font(font) -> returns index.
        // 2. WasmPage.render_layout(node, font, index).
        
        node.inner.render(&mut self.inner, area, &font.inner, font_index, &context);
    }
}

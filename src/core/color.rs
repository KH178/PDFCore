/// Color representation for PDF rendering (RGB/RGBA)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: f64,  // 0.0 to 1.0
    pub g: f64,
    pub b: f64,
    #[serde(default = "default_alpha")]
    pub a: f64,  // alpha (1.0 = opaque)
}

fn default_alpha() -> f64 { 1.0 }

impl Color {
    /// Create RGB color (opaque)
    pub fn rgb(r: f64, g: f64, b: f64) -> Self {
        Color { r, g, b, a: 1.0 }
    }
    
    /// Create RGBA color with transparency
    pub fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Color { r, g, b, a }
    }
    
    // Common colors
    pub fn black() -> Self { Color::rgb(0.0, 0.0, 0.0) }
    pub fn white() -> Self { Color::rgb(1.0, 1.0, 1.0) }
    pub fn red() -> Self { Color::rgb(1.0, 0.0, 0.0) }
    pub fn green() -> Self { Color::rgb(0.0, 1.0, 0.0) }
    pub fn blue() -> Self { Color::rgb(0.0, 0.0, 1.0) }
    pub fn gray(intensity: f64) -> Self { Color::rgb(intensity, intensity, intensity) }
    
    /// Convert to PDF fill color operator (rg)
    pub fn to_pdf_fill(&self) -> String {
        format!("{:.3} {:.3} {:.3} rg", self.r, self.g, self.b)
    }
    
    /// Convert to PDF stroke color operator (RG)
    pub fn to_pdf_stroke(&self) -> String {
        format!("{:.3} {:.3} {:.3} RG", self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::black()
    }
}

use std::sync::Arc;
use std::collections::HashSet;
use owned_ttf_parser::{OwnedFace, AsFaceRef};
use std::io::{self, Error, ErrorKind};

/// Represents a loaded font with parsing and shaping capabilities
#[derive(Clone)]
pub struct Font {
    pub(crate) face: Arc<OwnedFace>,
    pub(crate) name: String,
    pub(crate) units_per_em: u16,
}

impl Font {
    /// Load a font from a file path
    pub fn from_file(path: &str, name: String) -> io::Result<Self> {
        let data = std::fs::read(path)?;
        Self::from_bytes(data, name)
    }
    
    /// Load a font from bytes (e.g., embedded font data)
    pub fn from_bytes(data: Vec<u8>, name: String) -> io::Result<Self> {
        let face = OwnedFace::from_vec(data, 0)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid font file"))?;
        
        let units_per_em = face.as_face_ref().units_per_em();
        
        Ok(Font { 
            face: Arc::new(face), 
            name, 
            units_per_em 
        })
    }
    
    /// Shape text and return glyph IDs with positions
    pub fn shape_text(&self, text: &str, size: f64) -> Vec<ShapedGlyph> {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        
        // owned_ttf_parser uses Send+Sync, cloning Arc is fine
        let rb_face = rustybuzz::Face::from_face(self.face.as_face_ref().clone());
        let output = rustybuzz::shape(&rb_face, &[], buffer);
        
        let positions = output.glyph_positions();
        let infos = output.glyph_infos();
        
        let scale = size / self.units_per_em as f64;
        
        infos.iter().zip(positions.iter())
            .map(|(info, pos)| ShapedGlyph {
                glyph_id: info.glyph_id as u16,
                x_advance: pos.x_advance as f64 * scale,
                y_advance: pos.y_advance as f64 * scale,
                x_offset: pos.x_offset as f64 * scale,
                y_offset: pos.y_offset as f64 * scale,
            })
            .collect()
    }
    
    /// Measure text width using shaping
    pub fn measure_text(&self, text: &str, size: f64) -> f64 {
        let glyphs = self.shape_text(text, size);
        glyphs.iter().map(|g| g.x_advance).sum()
    }
    
    /// Get font metrics
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    
    /// Get raw font data for embedding
    pub fn get_font_data(&self) -> &[u8] {
        self.face.as_slice()
    }
    
    /// Get font ascender (scaled to 1000 units)
    pub fn ascent(&self) -> i16 {
        let face = self.face.as_face_ref();
        let ascender = face.ascender();
        (ascender as i32 * 1000 / self.units_per_em as i32) as i16
    }
    
    /// Get font descender (scaled to 1000 units)
    pub fn descent(&self) -> i16 {
        let face = self.face.as_face_ref();
        let descender = face.descender();
        (descender as i32 * 1000 / self.units_per_em as i32) as i16
    }
    
    /// Get font bbox
    pub fn bbox(&self) -> (i16, i16, i16, i16) {
        let face = self.face.as_face_ref();
        let bbox = face.global_bounding_box();
        let scale = 1000.0 / self.units_per_em as f32;
        (
            (bbox.x_min as f32 * scale) as i16,
            (bbox.y_min as f32 * scale) as i16,
            (bbox.x_max as f32 * scale) as i16,
            (bbox.y_max as f32 * scale) as i16,
        )
    }
    
    /// Get cap height (approximate if not available)
    pub fn cap_height(&self) -> i16 {
        let face = self.face.as_face_ref();
        if let Some(cap_height) = face.capital_height() {
            (cap_height as i32 * 1000 / self.units_per_em as i32) as i16
        } else {
            // Approximate as 70% of ascender
            (self.ascent() as i32 * 70 / 100) as i16
        }
    }
    
    /// Get italic angle
    pub fn italic_angle(&self) -> f32 {
        let face = self.face.as_face_ref();
        face.italic_angle()
    }
}

/// Represents a shaped glyph with position and advance information
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub glyph_id: u16,
    pub x_advance: f64,
    pub y_advance: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

/// Track which glyphs are used for font subsetting
#[derive(Default)]
pub struct GlyphUsage {
    used_gids: HashSet<u16>,
}

impl GlyphUsage {
    pub fn mark_used(&mut self, gid: u16) {
        self.used_gids.insert(gid);
    }
    
    pub fn is_used(&self, gid: u16) -> bool {
        self.used_gids.contains(&gid)
    }
    
    pub fn count(&self) -> usize {
        self.used_gids.len()
    }
}

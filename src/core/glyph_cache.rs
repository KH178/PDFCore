use std::collections::HashMap;
use rustybuzz::{Face, UnicodeBuffer};

/// Cache for shaped glyph runs to avoid re-shaping identical text
pub struct GlyphCache {
    cache: HashMap<GlyphCacheKey, Vec<GlyphInfo>>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct GlyphCacheKey {
    text: String,
    font_index: usize,
    size: u32,
}

#[derive(Clone)]
pub struct GlyphInfo {
    pub glyph_id: u16,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl GlyphCache {
    pub fn new() -> Self {
        GlyphCache {
            cache: HashMap::new(),
        }
    }
    
    /// Get shaped glyphs from cache or shape if not cached
    pub fn get_or_shape(
        &mut self,
        text: &str,
        font_index: usize,
        size: u32,
        face: &Face,
    ) -> Vec<GlyphInfo> {
        let key = GlyphCacheKey {
            text: text.to_string(),
            font_index,
            size,
        };
        
        if let Some(glyphs) = self.cache.get(&key) {
            return glyphs.clone();
        }
        
        // Shape the text
        let glyphs = shape_text(text, face, size);
        self.cache.insert(key, glyphs.clone());
        glyphs
    }
    
    /// Clear the cache (useful for memory management)
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        let entries = self.cache.len();
        let memory = std::mem::size_of::<GlyphCacheKey>() * entries
            + self.cache.values()
                .map(|v| v.len() * std::mem::size_of::<GlyphInfo>())
                .sum::<usize>();
        (entries, memory)
    }
}

/// Shape text using HarfBuzz
fn shape_text(text: &str, face: &Face, size: u32) -> Vec<GlyphInfo> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    
    let output = rustybuzz::shape(face, &[], buffer);
    let positions = output.glyph_positions();
    let infos = output.glyph_infos();
    
    let scale = size as f32 / face.units_per_em() as f32;
    
    infos
        .iter()
        .zip(positions.iter())
        .map(|(info, pos)| GlyphInfo {
            glyph_id: info.glyph_id as u16,
            x_advance: pos.x_advance as f32 * scale,
            y_advance: pos.y_advance as f32 * scale,
            x_offset: pos.x_offset as f32 * scale,
            y_offset: pos.y_offset as f32 * scale,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_hit() {
        let mut cache = GlyphCache::new();
        
        // First call should miss and shape
        // Second call should hit cache
        // (Would need actual font face to test properly)
    }
}

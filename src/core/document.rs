use std::collections::{HashMap, HashSet};
use std::io::{self, Error, ErrorKind, Write};
use crate::core::font::Font;
use crate::core::page::Page;
use crate::core::image::Image;
use crate::core::writer::{PdfWriter, PdfObject};
use flate2::write::ZlibEncoder;
use flate2::Compression;

/// Document operation mode
pub enum DocumentMode {
    /// Buffered mode: collect all pages in memory before writing
    Buffered(Vec<Page>),
    /// Streaming mode: write pages immediately as they're added
    Streaming {
        writer: PdfWriter,
        page_ids: Vec<u32>,
        next_object_id: u32,
        catalog_id: u32,
        pages_id: u32,
        font_id: u32,
        custom_font_ids: Vec<u32>,  // Track custom font object IDs
        image_ids: Vec<u32>,        // Track image object IDs (index -> object_id)
    },
}

/// Represents a PDF document with multiple pages
pub struct Document {
    pub mode: DocumentMode,
    pub fonts: Vec<Font>,  // Registered custom fonts
    pub fonts_embedded: bool,  // Track if fonts have been written in streaming mode
    pub images: Vec<Image>, // Registered images (Buffered mode only)
}

impl Document {
    /// Create a new empty document in buffered mode
    pub fn new() -> Self {
        Document {
            mode: DocumentMode::Buffered(Vec::new()),
            fonts: Vec::new(),
            fonts_embedded: false,
            images: Vec::new(),
        }
    }
    
    /// Create a new document in streaming mode
    /// Pages are written immediately as they're added
    pub fn streaming(path: &str) -> io::Result<Self> {
        let mut writer = PdfWriter::new(path)?;
        
        let catalog_id = 1;
        let pages_id = 2;
        let font_id = 3;
        let next_object_id = 4; // Next available object ID
        
        // Write Catalog (with forward reference to Pages)
        let catalog = PdfObject::Dictionary(vec![
            ("Type".to_string(), PdfObject::Name("Catalog".to_string())),
            ("Pages".to_string(), PdfObject::Reference(pages_id)),
        ]);
        writer.write_object(catalog_id, &catalog)?;
        
        // Write Font (shared resource)
        let font = PdfObject::Dictionary(vec![
            ("Type".to_string(), PdfObject::Name("Font".to_string())),
            ("Subtype".to_string(), PdfObject::Name("Type1".to_string())),
            ("BaseFont".to_string(), PdfObject::Name("Helvetica".to_string())),
        ]);
        writer.write_object(font_id, &font)?;
        
        Ok(Document {
            mode: DocumentMode::Streaming {
                writer,
                page_ids: Vec::new(),
                next_object_id,
                catalog_id,
                pages_id,
                font_id,
                custom_font_ids: Vec::new(),
                image_ids: Vec::new(),
            },
            fonts: Vec::new(),
            fonts_embedded: false,
            images: Vec::new(),
        })
    }
    
    /// Register a custom font with the document
    /// Returns the font index to use in page rendering
    pub fn add_font(&mut self, font: &Font) -> u32 {
        self.fonts.push(font.clone());
        (self.fonts.len() - 1) as u32
    }

    /// Register an image with the document
    /// Returns the image index to use in page rendering
    pub fn add_image(&mut self, image: &Image) -> io::Result<u32> {
        match &mut self.mode {
            DocumentMode::Buffered(_) => {
                self.images.push(image.clone());
                Ok((self.images.len() - 1) as u32)
            }
            DocumentMode::Streaming { 
                writer, 
                next_object_id, 
                image_ids, 
                .. 
            } => {
                // In streaming mode, write image object immediately
                let image_id = *next_object_id;
                embed_image(writer, image, image_id)?;
                
                *next_object_id += 1;
                image_ids.push(image_id);
                
                Ok((image_ids.len() - 1) as u32)
            }
        }
    }
    
    /// Add a page to the document
    pub fn add_page(&mut self, page: &Page) -> io::Result<()> {
        let page = page.clone(); // Page is Clone
        match &mut self.mode {
            DocumentMode::Buffered(pages) => {
                pages.push(page);
                Ok(())
            }
            DocumentMode::Streaming {
                writer,
                page_ids,
                next_object_id,
                pages_id,
                font_id,
                custom_font_ids,
                image_ids,
                ..  // Ignore catalog_id
            } => {
                // Embed fonts lazily before the first page
                if !self.fonts_embedded && !self.fonts.is_empty() {
                    for font in &self.fonts {
                        let base_id = *next_object_id;
                        let type0_id = embed_custom_font(writer, font, base_id, None)?;
                        custom_font_ids.push(type0_id);
                        *next_object_id += 4;  // 4 objects per font
                    }
                    self.fonts_embedded = true;
                }
                
                // Write content stream immediately
                let content_id = *next_object_id;
                *next_object_id += 1;
                
                let content_stream = PdfObject::Stream(vec![], page.content.clone());
                writer.write_object(content_id, &content_stream)?;
                
                // Build font resources dictionary including custom fonts
                let mut font_resources = vec![
                    ("F1".to_string(), PdfObject::Reference(*font_id))
                ];
                for (i, type0_id) in custom_font_ids.iter().enumerate() {
                    font_resources.push((format!("F{}", i + 2), PdfObject::Reference(*type0_id)));
                }

                // Build XObject resources (images)
                let mut xobject_resources = Vec::new();
                for image_idx in &page.used_images {
                    if let Some(obj_id) = image_ids.get(*image_idx as usize) {
                        xobject_resources.push((format!("Im{}", image_idx), PdfObject::Reference(*obj_id)));
                    }
                }
                
                let mut resources_dict = vec![
                    ("Font".to_string(), PdfObject::Dictionary(font_resources))
                ];
                if !xobject_resources.is_empty() {
                    resources_dict.push(("XObject".to_string(), PdfObject::Dictionary(xobject_resources)));
                }

                // Write page object immediately
                let page_id = *next_object_id;
                *next_object_id += 1;
                
                let page_obj = PdfObject::Dictionary(vec![
                    ("Type".to_string(), PdfObject::Name("Page".to_string())),
                    ("Parent".to_string(), PdfObject::Reference(*pages_id)),
                    ("MediaBox".to_string(), PdfObject::Array(vec![
                        PdfObject::Integer(0),
                        PdfObject::Integer(0),
                        PdfObject::Real(page.width as f64),
                        PdfObject::Real(page.height as f64),
                    ])),
                    ("Resources".to_string(), PdfObject::Dictionary(resources_dict)),
                    ("Contents".to_string(), PdfObject::Reference(content_id)),
                ]);
                writer.write_object(page_id, &page_obj)?;
                
                // Track page ID for later
                page_ids.push(page_id);
                
                Ok(())
            }
        }
    }
    
    /// Finalize a streaming document
    /// Only for streaming mode - writes the Pages tree and xref/trailer
    pub fn finalize(&mut self) -> io::Result<()> {
        match &mut self.mode {
            DocumentMode::Buffered(_) => {
                Err(Error::new(ErrorKind::Other, "finalize() is only for streaming mode. Use write_to() for buffered mode."))
            }
            DocumentMode::Streaming {
                writer,
                page_ids,
                pages_id,
                catalog_id,
                ..
            } => {
                // Now write the Pages object with all Kids
                let page_refs: Vec<PdfObject> = page_ids.iter()
                    .map(|page_id| PdfObject::Reference(*page_id))
                    .collect();
                
                let pages = PdfObject::Dictionary(vec![
                    ("Type".to_string(), PdfObject::Name("Pages".to_string())),
                    ("Kids".to_string(), PdfObject::Array(page_refs)),
                    ("Count".to_string(), PdfObject::Integer(page_ids.len() as i64)),
                ]);
                writer.write_object(*pages_id, &pages)?;
                
                // Write xref and trailer
                writer.write_xref_and_trailer(*catalog_id)?;
                
                Ok(())
            }
        }
    }
    
    /// Write the document to a file (buffered mode)
    pub fn write_to(&self, path: &str) -> io::Result<()> {
        match &self.mode {
            DocumentMode::Streaming { .. } => {
                Err(Error::new(ErrorKind::Other, "write_to() is only for buffered mode. Use finalize() for streaming mode."))
            }
            DocumentMode::Buffered(pages) => {
                let mut writer = PdfWriter::new(path)?;
                
                let catalog_id = 1;
                let pages_id = 2;
                let font_id = 3;  // Built-in Helvetica
                
                // Calculate object IDs for custom fonts (each font needs 4 objects)
                let mut custom_font_ids = Vec::new();
                let mut next_id = 4;
                for _ in 0..self.fonts.len() {
                    custom_font_ids.push(next_id);
                    next_id += 4;  // FontFile, FontDescriptor, CIDFont, Type0
                }

                // Calculate object IDs for images
                let mut image_object_ids = Vec::new();
                for _ in 0..self.images.len() {
                    image_object_ids.push(next_id);
                    next_id += 1;
                }
                
                // Calculate object IDs for pages
                let mut page_object_ids = Vec::new();
                for i in 0..pages.len() {
                    let content_id = next_id + (i * 2) as u32;
                    let page_id = next_id + 1 + (i * 2) as u32;
                    page_object_ids.push((content_id, page_id));
                }
                
                // Write Catalog
                let catalog = PdfObject::Dictionary(vec![
                    ("Type".to_string(), PdfObject::Name("Catalog".to_string())),
                    ("Pages".to_string(), PdfObject::Reference(pages_id)),
                ]);
                writer.write_object(catalog_id, &catalog)?;
                
                // Write Pages tree
                let page_refs: Vec<PdfObject> = page_object_ids.iter()
                    .map(|(_content_id, page_id)| PdfObject::Reference(*page_id))
                    .collect();
                
                let pages_obj = PdfObject::Dictionary(vec![
                    ("Type".to_string(), PdfObject::Name("Pages".to_string())),
                    ("Kids".to_string(), PdfObject::Array(page_refs)),
                    ("Count".to_string(), PdfObject::Integer(pages.len() as i64)),
                ]);
                writer.write_object(pages_id, &pages_obj)?;
                
                // Write built-in Helvetica font
                let font = PdfObject::Dictionary(vec![
                    ("Type".to_string(), PdfObject::Name("Font".to_string())),
                    ("Subtype".to_string(), PdfObject::Name("Type1".to_string())),
                    ("BaseFont".to_string(), PdfObject::Name("Helvetica".to_string())),
                ]);
                writer.write_object(font_id, &font)?;
                
                // Aggregate glyph usage across all pages for subsetting
                let mut font_glyph_usage: HashMap<usize, HashSet<u16>> = HashMap::new();
                for page in pages {
                    for (font_idx, gids) in &page.used_glyphs {
                        font_glyph_usage
                            .entry(*font_idx)
                            .or_insert_with(HashSet::new)
                            .extend(gids);
                    }
                }
                
                // Embed custom fonts with subsetting
                let mut type0_font_ids = Vec::new();
                for (i, font) in self.fonts.iter().enumerate() {
                    let used_gids = font_glyph_usage.get(&i);
                    let type0_id = embed_custom_font(&mut writer, font, custom_font_ids[i], used_gids)?;
                    type0_font_ids.push(type0_id);
                }

                // Embed images
                for (i, image) in self.images.iter().enumerate() {
                    embed_image(&mut writer, image, image_object_ids[i])?;
                }
                
                // Build font resources dictionary
                let mut font_resources = vec![
                    ("F1".to_string(), PdfObject::Reference(font_id))
                ];
                for (i, type0_id) in type0_font_ids.iter().enumerate() {
                    font_resources.push((format!("F{}", i + 2), PdfObject::Reference(*type0_id)));
                }
                
                // Write each page
                for (i, page) in pages.iter().enumerate() {
                    let (content_id, page_id) = page_object_ids[i];
                    
                    let content_stream = PdfObject::Stream(vec![], page.content.clone());
                    writer.write_object(content_id, &content_stream)?;

                    // Build XObject resources (images)
                    let mut xobject_resources = Vec::new();
                    for image_idx in &page.used_images {
                        if let Some(obj_id) = image_object_ids.get(*image_idx as usize) {
                            xobject_resources.push((format!("Im{}", image_idx), PdfObject::Reference(*obj_id)));
                        }
                    }

                    let mut resources_dict = vec![
                        ("Font".to_string(), PdfObject::Dictionary(font_resources.clone()))
                    ];
                    if !xobject_resources.is_empty() {
                        resources_dict.push(("XObject".to_string(), PdfObject::Dictionary(xobject_resources)));
                    }
                    
                    let page_obj = PdfObject::Dictionary(vec![
                        ("Type".to_string(), PdfObject::Name("Page".to_string())),
                        ("Parent".to_string(), PdfObject::Reference(pages_id)),
                        ("MediaBox".to_string(), PdfObject::Array(vec![
                            PdfObject::Integer(0),
                            PdfObject::Integer(0),
                            PdfObject::Real(page.width as f64),
                            PdfObject::Real(page.height as f64),
                        ])),
                        ("Resources".to_string(), PdfObject::Dictionary(resources_dict)),
                        ("Contents".to_string(), PdfObject::Reference(content_id)),
                    ]);
                    writer.write_object(page_id, &page_obj)?;
                }
                
                writer.write_xref_and_trailer(catalog_id)?;
                
                Ok(())
            }
        }
    }
}

/// Subset a font to include only used glyphs
fn subset_font(font: &Font, used_gids: &HashSet<u16>) -> Vec<u8> {
    let font_data = font.get_font_data();
    let mut gids: Vec<u16> = used_gids.iter().copied().collect();
    gids.sort();
    let profile = subsetter::Profile::pdf(&gids);
    match subsetter::subset(font_data, 0, profile) {
        Ok(subset_data) => subset_data,
        Err(e) => {
            eprintln!("Warning: Font subsetting failed ({:?}), using full font", e);
            font_data.to_vec()
        }
    }
}

/// Embed a custom TrueType font into PDF
fn embed_custom_font(writer: &mut PdfWriter, font: &Font, base_id: u32, used_gids: Option<&HashSet<u16>>) -> io::Result<u32> {
    let font_file_id = base_id;
    let font_descriptor_id = base_id + 1;
    let cid_font_id = base_id + 2;
    let type0_font_id = base_id + 3;
    
    // 1. Write TrueType font file stream
    let font_data = if let Some(gids) = used_gids {
        subset_font(font, gids)
    } else {
        font.get_font_data().to_vec()
    };
    
    let font_file = PdfObject::Stream(
        vec![("Length1".to_string(), PdfObject::Integer(font_data.len() as i64))],
        font_data
    );
    writer.write_object(font_file_id, &font_file)?;
    
    // 2. Write FontDescriptor
    let bbox = font.bbox();
    let font_descriptor = PdfObject::Dictionary(vec![
        ("Type".to_string(), PdfObject::Name("FontDescriptor".to_string())),
        ("FontName".to_string(), PdfObject::Name(font.get_name().to_string())),
        ("Flags".to_string(), PdfObject::Integer(4)), // 4 = Symbolic (required for Identity-H)
        ("FontBBox".to_string(), PdfObject::Array(vec![
            PdfObject::Integer(bbox.0 as i64),
            PdfObject::Integer(bbox.1 as i64),
            PdfObject::Integer(bbox.2 as i64),
            PdfObject::Integer(bbox.3 as i64),
        ])),
        ("ItalicAngle".to_string(), PdfObject::Real(font.italic_angle() as f64)),
        ("Ascent".to_string(), PdfObject::Integer(font.ascent() as i64)),
        ("Descent".to_string(), PdfObject::Integer(font.descent() as i64)),
        ("CapHeight".to_string(), PdfObject::Integer(font.cap_height() as i64)),
        ("StemV".to_string(), PdfObject::Integer(80)),
        ("FontFile2".to_string(), PdfObject::Reference(font_file_id)),
    ]);
    writer.write_object(font_descriptor_id, &font_descriptor)?;
    
    // Generate W array (Widths)
    let w_array = if let Some(gids) = used_gids {
        // Sort GIDs to produce compact ranges if possible
        // For MVP, just output [ gid [ width ] ] for each used GID?
        // Better: [ 0 [ w0 w1 w2 ... ] ] if contiguous, but Identity mapping is sparse if subsetted?
        // If subsetted with Identity map, the CIDs ARE the GIDs.
        // So we need to specify widths for the sparse CIDs.
        // Format: [ c [w] c [w] ... ] is inefficient.
        // Format: [ first_cid [ w1 w2 ... ] ]
        // Since we know the used GIDs, we can iterate them in order.
        let mut sorted_gids: Vec<u16> = gids.iter().copied().collect();
        sorted_gids.sort();
        
        let mut w = Vec::new();
        // Naive approach: individual entries [ cid [width] ]
        // Optimizing for ranges is better but more complex.
        // Let's optimize slightly: group contiguous ranges.
        // OR: PDF allows: c [w ...].
        
        // Actually, if we use subsetting, we only have a few glyphs.
        // But if we disable subsetting, we have thousands.
        // Let's stick to individual check for now, optimizing later if slow.
        // Wait, for full font, writing 65k entries is confusing.
        // Ideally we should use the font's hmtx table.
        // But for Identity-H, we just need W for CIDs we use.
        
        for gid in sorted_gids {
            let width = font.get_glyph_width(gid);
            // Scaling: UnitsPerEm -> 1000
            let scale = 1000.0 / font.units_per_em() as f32;
            let pdf_width = (width as f32 * scale) as i64;
            
            w.push(PdfObject::Integer(gid as i64));
            w.push(PdfObject::Array(vec![PdfObject::Integer(pdf_width)]));
        }
        PdfObject::Array(w)
    } else {
        // Streaming mode: we must provide widths for ALL glyphs since we don't know usage.
        // We cannot use subsetting, so we embedded the full font.
        // Now we must provide the W array for the full font.
        // Ideally we should compress this (use ranges), but for now, let's output a single block for all glyphs.
        // This is large but correct.
        
        
        let num_glyphs = font.number_of_glyphs();
        let scale = 1000.0 / font.units_per_em() as f32;
        let mut widths = Vec::with_capacity(num_glyphs as usize);
        
        for gid in 0..num_glyphs {
            let width = font.get_glyph_width(gid);
            let pdf_width = (width as f32 * scale) as i64;
            widths.push(PdfObject::Integer(pdf_width));
        }
        
        // Format: [ 0 [ w0 w1 ... wn ] ] 
        // Start at CID 0, provide array of all widths
        PdfObject::Array(vec![
            PdfObject::Integer(0),
            PdfObject::Array(widths)
        ])
    };

    //3. Write CIDFont
    let cid_font = PdfObject::Dictionary(vec![
        ("Type".to_string(), PdfObject::Name("Font".to_string())),
        ("Subtype".to_string(), PdfObject::Name("CIDFontType2".to_string())),
        ("BaseFont".to_string(), PdfObject::Name(font.get_name().to_string())),
        ("CIDSystemInfo".to_string(), PdfObject::Dictionary(vec![
            ("Registry".to_string(), PdfObject::String("Adobe".to_string())),
            ("Ordering".to_string(), PdfObject::String("Identity".to_string())),
            ("Supplement".to_string(), PdfObject::Integer(0)),
        ])),
        ("FontDescriptor".to_string(), PdfObject::Reference(font_descriptor_id)),
        ("CIDToGIDMap".to_string(), PdfObject::Name("Identity".to_string())),
        ("DW".to_string(), PdfObject::Integer(1000)),
        ("W".to_string(), w_array),
    ]);
    writer.write_object(cid_font_id, &cid_font)?;
    
    // 4. Write Type0 composite font
    let type0_font = PdfObject::Dictionary(vec![
        ("Type".to_string(), PdfObject::Name("Font".to_string())),
        ("Subtype".to_string(), PdfObject::Name("Type0".to_string())),
        ("BaseFont".to_string(), PdfObject::Name(font.get_name().to_string())),
        ("Encoding".to_string(), PdfObject::Name("Identity-H".to_string())),
        ("DescendantFonts".to_string(), PdfObject::Array(vec![
            PdfObject::Reference(cid_font_id)
        ])),
    ]);
    writer.write_object(type0_font_id, &type0_font)?;
    
    Ok(type0_font_id)
}

/// Embed an image into the PDF
fn embed_image(writer: &mut PdfWriter, image: &Image, object_id: u32) -> io::Result<()> {
    // If filter is explicitly set (e.g. DCTDecode for JPEG), use raw data
    // If filter is None or FlateDecode was requested (for PNG), compress data
    
    let (data, filter) = if let Some(f) = &image.filter {
        if f == "FlateDecode" {
            // Re-compress using Flate (zlib)
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&image.data)?;
            (encoder.finish()?, Some("FlateDecode".to_string()))
        } else {
            // Passthrough (e.g. JPEG)
            (image.data.clone(), Some(f.clone()))
        }
    } else {
        // Default to Flate
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&image.data)?;
        (encoder.finish()?, Some("FlateDecode".to_string()))
    };

    let mut dict = vec![
        ("Type".to_string(), PdfObject::Name("XObject".to_string())),
        ("Subtype".to_string(), PdfObject::Name("Image".to_string())),
        ("Width".to_string(), PdfObject::Integer(image.width as i64)),
        ("Height".to_string(), PdfObject::Integer(image.height as i64)),
        ("ColorSpace".to_string(), PdfObject::Name(image.color_space.clone())),
        ("BitsPerComponent".to_string(), PdfObject::Integer(image.bits_per_component as i64)),
    ];
    
    if let Some(f) = filter {
        dict.push(("Filter".to_string(), PdfObject::Name(f)));
    }
    
    let image_obj = PdfObject::Stream(dict, data);
    writer.write_object(object_id, &image_obj)?;
    
    Ok(())
}

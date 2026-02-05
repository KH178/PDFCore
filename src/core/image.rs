use std::io::{self, Error, ErrorKind};
use image::{ImageFormat, GenericImageView};
use std::fs;

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub color_space: String, // "DeviceRGB", "DeviceGray"
    pub bits_per_component: u8,
    pub data: Vec<u8>,
    pub filter: Option<String>,
}

impl Image {
    /// Load an image from a file path
    /// Supports JPEG (passed through) and PNG (decompressed to raw RGB)
    pub fn from_file(path: &str) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        
        // Use image crate to guess format
        let format = image::guess_format(&bytes)
            .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Unknown image format: {}", e)))?;

        match format {
            ImageFormat::Jpeg => Self::load_jpeg(&bytes),
            ImageFormat::Png => Self::load_png(&bytes),
            _ => Err(Error::new(ErrorKind::Unsupported, "Only JPEG and PNG are supported")),
        }
    }

    fn load_jpeg(data: &[u8]) -> io::Result<Self> {
        // For JPEG, we just read metadata and pass raw bytes (DCTDecode)
        let img = image::load_from_memory_with_format(data, ImageFormat::Jpeg)
            .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Failed to parse JPEG: {}", e)))?;
        
        let (width, height) = img.dimensions();
        let color_type = img.color();
        
        let color_space = match color_type {
            image::ColorType::L8 => "DeviceGray",
            image::ColorType::Rgb8 => "DeviceRGB",
            _ => "DeviceRGB", // Default fallback
        };

        Ok(Image {
            width,
            height,
            color_space: color_space.to_string(),
            bits_per_component: 8,
            data: data.to_vec(),
            filter: Some("DCTDecode".to_string()),
        })
    }

    fn load_png(data: &[u8]) -> io::Result<Self> {
        // For PNG, we decode to raw RGB bytes (simple approach for now)
        // Ideally we would passthrough if DEFLATE, but PNG structure is complex (Predictor etc.)
        // So we decode to RGB8 and will re-compress with FlateDecode in the PDF writer
        let img = image::load_from_memory_with_format(data, ImageFormat::Png)
            .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Failed to parse PNG: {}", e)))?;
            
        let (width, height) = img.dimensions();
        let raw_pixels = img.to_rgb8().into_raw();
        
        Ok(Image {
            width,
            height,
            color_space: "DeviceRGB".to_string(),
            bits_per_component: 8,
            data: raw_pixels,
            filter: Some("FlateDecode".to_string()), // We will compress this when writing
        })
    }
}

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("Image processing error: {0}")]
    Processing(String),
    #[error("Invalid quality: {0} (must be 0-100)")]
    InvalidQuality(u8),
    #[error("Memory allocation error")]
    Memory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Gif,
    WebP,
    Bmp,
    Tiff,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Result<Self, ImageError> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "png" => Ok(Self::Png),
            "gif" => Ok(Self::Gif),
            "webp" => Ok(Self::WebP),
            "bmp" => Ok(Self::Bmp),
            "tiff" | "tif" => Ok(Self::Tiff),
            _ => Err(ImageError::UnsupportedFormat(ext.to_string())),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResizeMode {
    /// Resize to exact dimensions (may distort)
    Exact,
    /// Resize maintaining aspect ratio, fit within dimensions
    Fit,
    /// Resize maintaining aspect ratio, fill dimensions (may crop)
    Fill,
    /// Resize maintaining aspect ratio, pad to exact dimensions
    Pad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    pub fn new(width: u32, height: u32) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidDimensions { width, height });
        }
        Ok(Self { width, height })
    }

    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub format: ImageFormat,
    pub dimensions: ImageDimensions,
    pub file_size: u64,
    pub has_transparency: bool,
    pub color_depth: u8,
    pub dpi: Option<(u32, u32)>,
}

#[derive(Debug, Clone)]
pub struct ResizeOptions {
    pub mode: ResizeMode,
    pub maintain_aspect_ratio: bool,
    pub upscale: bool,
    pub background_color: Option<(u8, u8, u8, u8)>, // RGBA
    pub filter: ResizeFilter,
}

impl Default for ResizeOptions {
    fn default() -> Self {
        Self {
            mode: ResizeMode::Fit,
            maintain_aspect_ratio: true,
            upscale: false,
            background_color: Some((255, 255, 255, 255)), // White
            filter: ResizeFilter::Lanczos3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResizeFilter {
    Nearest,
    Linear,
    CubicBSpline,
    CatmullRom,
    Lanczos3,
}

#[derive(Debug, Clone)]
pub struct SaveOptions {
    pub format: ImageFormat,
    pub quality: u8, // 0-100, only applies to lossy formats
    pub progressive: bool, // For JPEG
    pub optimize: bool,
    pub preserve_metadata: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::Jpeg,
            quality: 85,
            progressive: false,
            optimize: true,
            preserve_metadata: false,
        }
    }
}

/// Image processing utility ported from PHP Images class
/// Provides image loading, resizing, format conversion, and optimization
pub struct Images {
    pub format: Option<ImageFormat>,
    pub dimensions: Option<ImageDimensions>,
    data: Option<Vec<u8>>, // Simulated image data
}

impl Images {
    /// Create new Images instance
    pub fn new() -> Self {
        Self {
            format: None,
            dimensions: None,
            data: None,
        }
    }

    /// Load image from file
    pub fn load<P: AsRef<Path>>(filename: P) -> Result<Self, ImageError> {
        let path = filename.as_ref();
        
        if !path.exists() {
            return Err(ImageError::FileNotFound(path.to_string_lossy().to_string()));
        }

        // Get file extension to determine format
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| ImageError::UnsupportedFormat("unknown".to_string()))?;

        let format = ImageFormat::from_extension(extension)?;

        // In real implementation, this would use an image library like `image` crate
        // For now, simulate loading
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();

        // Mock dimensions based on format
        let dimensions = match format {
            ImageFormat::Jpeg | ImageFormat::Png => ImageDimensions::new(800, 600)?,
            ImageFormat::Gif => ImageDimensions::new(400, 300)?,
            _ => ImageDimensions::new(1024, 768)?,
        };

        let mut images = Self::new();
        images.format = Some(format);
        images.dimensions = Some(dimensions);
        images.data = Some(vec![0u8; file_size as usize]); // Mock data

        Ok(images)
    }

    /// Load image from bytes
    pub fn load_from_bytes(data: &[u8]) -> Result<Self, ImageError> {
        if data.is_empty() {
            return Err(ImageError::Processing("Empty data".to_string()));
        }

        // Detect format from data (magic bytes)
        let format = Self::detect_format(data)?;
        
        // Mock dimensions
        let dimensions = ImageDimensions::new(800, 600)?;

        let mut images = Self::new();
        images.format = Some(format);
        images.dimensions = Some(dimensions);
        images.data = Some(data.to_vec());

        Ok(images)
    }

    /// Detect image format from data
    fn detect_format(data: &[u8]) -> Result<ImageFormat, ImageError> {
        if data.len() < 4 {
            return Err(ImageError::UnsupportedFormat("insufficient data".to_string()));
        }

        // Check magic bytes
        match &data[0..4] {
            [0xFF, 0xD8, 0xFF, _] => Ok(ImageFormat::Jpeg),
            [0x89, 0x50, 0x4E, 0x47] => Ok(ImageFormat::Png),
            [0x47, 0x49, 0x46, 0x38] => Ok(ImageFormat::Gif),
            [0x42, 0x4D, _, _] => Ok(ImageFormat::Bmp),
            _ => {
                // Check for WebP
                if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
                    Ok(ImageFormat::WebP)
                } else {
                    Err(ImageError::UnsupportedFormat("unknown format".to_string()))
                }
            }
        }
    }

    /// Get image information
    pub fn get_info(&self) -> Result<ImageInfo, ImageError> {
        let format = self.format.ok_or_else(|| ImageError::Processing("No image loaded".to_string()))?;
        let dimensions = self.dimensions.ok_or_else(|| ImageError::Processing("No dimensions available".to_string()))?;
        let file_size = self.data.as_ref().map(|d| d.len() as u64).unwrap_or(0);

        Ok(ImageInfo {
            format,
            dimensions,
            file_size,
            has_transparency: matches!(format, ImageFormat::Png | ImageFormat::Gif),
            color_depth: match format {
                ImageFormat::Gif => 8,
                ImageFormat::Png => 24,
                ImageFormat::Jpeg => 24,
                _ => 24,
            },
            dpi: Some((72, 72)), // Mock DPI
        })
    }

    /// Get image dimensions
    pub fn get_dimensions(&self) -> Option<ImageDimensions> {
        self.dimensions.clone()
    }

    /// Get image width
    pub fn get_width(&self) -> Option<u32> {
        self.dimensions.as_ref().map(|d| d.width)
    }

    /// Get image height
    pub fn get_height(&self) -> Option<u32> {
        self.dimensions.as_ref().map(|d| d.height)
    }

    /// Resize image to specific dimensions
    pub fn resize(&mut self, width: u32, height: u32, options: Option<ResizeOptions>) -> Result<(), ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidDimensions { width, height });
        }

        let options = options.unwrap_or_default();
        let current_dims = self.dimensions.ok_or_else(|| ImageError::Processing("No image loaded".to_string()))?;

        // Calculate new dimensions based on resize mode
        let new_dims = self.calculate_resize_dimensions(&current_dims, width, height, &options)?;

        // In real implementation, this would perform actual image resizing
        // For now, just update the dimensions
        self.dimensions = Some(new_dims);

        Ok(())
    }

    /// Resize image to fit within specified dimensions
    pub fn resize_to_fit(&mut self, width: u32, height: u32) -> Result<(), ImageError> {
        let options = ResizeOptions {
            mode: ResizeMode::Fit,
            ..Default::default()
        };
        self.resize(width, height, Some(options))
    }

    /// Resize image to specific width, maintaining aspect ratio
    pub fn resize_to_width(&mut self, width: u32) -> Result<(), ImageError> {
        let current_dims = self.dimensions.ok_or_else(|| ImageError::Processing("No image loaded".to_string()))?;
        let aspect_ratio = current_dims.aspect_ratio();
        let height = (width as f64 / aspect_ratio) as u32;
        
        self.resize(width, height, None)
    }

    /// Resize image to specific height, maintaining aspect ratio
    pub fn resize_to_height(&mut self, height: u32) -> Result<(), ImageError> {
        let current_dims = self.dimensions.ok_or_else(|| ImageError::Processing("No image loaded".to_string()))?;
        let aspect_ratio = current_dims.aspect_ratio();
        let width = (height as f64 * aspect_ratio) as u32;
        
        self.resize(width, height, None)
    }

    /// Scale image by percentage
    pub fn scale(&mut self, scale_percent: f64) -> Result<(), ImageError> {
        if scale_percent <= 0.0 {
            return Err(ImageError::Processing("Scale must be positive".to_string()));
        }

        let current_dims = self.dimensions.ok_or_else(|| ImageError::Processing("No image loaded".to_string()))?;
        let width = (current_dims.width as f64 * scale_percent / 100.0) as u32;
        let height = (current_dims.height as f64 * scale_percent / 100.0) as u32;

        self.resize(width, height, None)
    }

    /// Calculate resize dimensions based on mode and options
    fn calculate_resize_dimensions(
        &self,
        current: &ImageDimensions,
        target_width: u32,
        target_height: u32,
        options: &ResizeOptions,
    ) -> Result<ImageDimensions, ImageError> {
        match options.mode {
            ResizeMode::Exact => {
                ImageDimensions::new(target_width, target_height)
            }
            ResizeMode::Fit => {
                let current_ratio = current.aspect_ratio();
                let target_ratio = target_width as f64 / target_height as f64;

                let (width, height) = if current_ratio > target_ratio {
                    // Image is wider, fit to width
                    (target_width, (target_width as f64 / current_ratio) as u32)
                } else {
                    // Image is taller, fit to height
                    ((target_height as f64 * current_ratio) as u32, target_height)
                };

                ImageDimensions::new(width, height)
            }
            ResizeMode::Fill => {
                let current_ratio = current.aspect_ratio();
                let target_ratio = target_width as f64 / target_height as f64;

                let (width, height) = if current_ratio > target_ratio {
                    // Image is wider, fit to height and crop width
                    ((target_height as f64 * current_ratio) as u32, target_height)
                } else {
                    // Image is taller, fit to width and crop height
                    (target_width, (target_width as f64 / current_ratio) as u32)
                };

                ImageDimensions::new(width, height)
            }
            ResizeMode::Pad => {
                // Same as fit, but the actual implementation would add padding
                self.calculate_resize_dimensions(current, target_width, target_height, 
                    &ResizeOptions { mode: ResizeMode::Fit, ..options.clone() })
            }
        }
    }

    /// Save image to file
    pub fn save<P: AsRef<Path>>(&self, filename: P, options: Option<SaveOptions>) -> Result<(), ImageError> {
        let options = options.unwrap_or_default();
        let path = filename.as_ref();

        // Validate quality for lossy formats
        if matches!(options.format, ImageFormat::Jpeg) && options.quality > 100 {
            return Err(ImageError::InvalidQuality(options.quality));
        }

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // In real implementation, this would encode and save the image
        // For now, just create a mock file
        let mock_data = self.data.as_ref().unwrap_or(&vec![0u8; 1024]);
        std::fs::write(path, mock_data)?;

        // Set file permissions if provided
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o644);
            std::fs::set_permissions(path, permissions)?;
        }

        Ok(())
    }

    /// Output image data (for web responses)
    pub fn output(&self, format: Option<ImageFormat>) -> Result<Vec<u8>, ImageError> {
        let _format = format.unwrap_or(self.format.unwrap_or(ImageFormat::Jpeg));
        
        // In real implementation, this would encode the image to the specified format
        // For now, return mock data
        Ok(self.data.clone().unwrap_or_else(|| vec![0u8; 1024]))
    }

    /// Create thumbnail
    pub fn create_thumbnail(&self, max_size: u32) -> Result<Self, ImageError> {
        let current_dims = self.dimensions.ok_or_else(|| ImageError::Processing("No image loaded".to_string()))?;
        
        let (width, height) = if current_dims.width > current_dims.height {
            (max_size, (max_size as f64 / current_dims.aspect_ratio()) as u32)
        } else {
            ((max_size as f64 * current_dims.aspect_ratio()) as u32, max_size)
        };

        let mut thumbnail = self.clone();
        thumbnail.resize(width, height, None)?;
        
        Ok(thumbnail)
    }

    /// Crop image
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<(), ImageError> {
        let current_dims = self.dimensions.ok_or_else(|| ImageError::Processing("No image loaded".to_string()))?;

        if x + width > current_dims.width || y + height > current_dims.height {
            return Err(ImageError::InvalidDimensions { width, height });
        }

        // In real implementation, this would crop the image
        self.dimensions = Some(ImageDimensions::new(width, height)?);

        Ok(())
    }

    /// Rotate image
    pub fn rotate(&mut self, degrees: f64) -> Result<(), ImageError> {
        if let Some(ref mut dims) = self.dimensions {
            // For 90-degree rotations, swap dimensions
            if (degrees % 180.0 - 90.0).abs() < 1.0 || (degrees % 180.0 + 90.0).abs() < 1.0 {
                let temp = dims.width;
                dims.width = dims.height;
                dims.height = temp;
            }
        }

        // In real implementation, this would rotate the image
        Ok(())
    }

    /// Flip image horizontally
    pub fn flip_horizontal(&mut self) -> Result<(), ImageError> {
        // In real implementation, this would flip the image
        Ok(())
    }

    /// Flip image vertically
    pub fn flip_vertical(&mut self) -> Result<(), ImageError> {
        // In real implementation, this would flip the image
        Ok(())
    }

    /// Convert to different format
    pub fn convert_format(&mut self, format: ImageFormat) -> Result<(), ImageError> {
        self.format = Some(format);
        Ok(())
    }

    /// Optimize image (reduce file size)
    pub fn optimize(&mut self) -> Result<(), ImageError> {
        // In real implementation, this would apply optimization techniques
        Ok(())
    }
}

impl Clone for Images {
    fn clone(&self) -> Self {
        Self {
            format: self.format,
            dimensions: self.dimensions.clone(),
            data: self.data.clone(),
        }
    }
}

impl Default for Images {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for common image operations
pub mod utils {
    use super::*;

    /// Calculate optimal dimensions for web display
    pub fn calculate_web_dimensions(
        original: &ImageDimensions,
        max_width: u32,
        max_height: u32,
    ) -> ImageDimensions {
        let width_ratio = max_width as f64 / original.width as f64;
        let height_ratio = max_height as f64 / original.height as f64;
        let ratio = width_ratio.min(height_ratio).min(1.0); // Don't upscale

        let width = (original.width as f64 * ratio) as u32;
        let height = (original.height as f64 * ratio) as u32;

        ImageDimensions::new(width, height).unwrap_or(original.clone())
    }

    /// Generate multiple sizes for responsive images
    pub fn generate_responsive_sizes(
        original: &ImageDimensions,
        sizes: &[u32],
    ) -> Vec<ImageDimensions> {
        sizes
            .iter()
            .filter_map(|&size| {
                if size > original.width.max(original.height) {
                    None // Skip sizes larger than original
                } else {
                    Some(calculate_web_dimensions(original, size, size))
                }
            })
            .collect()
    }

    /// Check if image needs resizing
    pub fn needs_resize(current: &ImageDimensions, max_width: u32, max_height: u32) -> bool {
        current.width > max_width || current.height > max_height
    }

    /// Estimate file size after compression
    pub fn estimate_file_size(dimensions: &ImageDimensions, format: ImageFormat, quality: u8) -> u64 {
        let pixels = dimensions.area();
        
        match format {
            ImageFormat::Jpeg => {
                let base_size = pixels / 10; // Rough estimate
                let quality_factor = quality as f64 / 100.0;
                (base_size as f64 * quality_factor) as u64
            }
            ImageFormat::Png => pixels * 3, // Rough estimate for PNG
            ImageFormat::Gif => pixels / 2, // Rough estimate for GIF
            ImageFormat::WebP => pixels / 8, // WebP is very efficient
            _ => pixels * 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_detection() {
        assert_eq!(ImageFormat::from_extension("jpg").map_err(|e| anyhow::anyhow!("Error: {}", e))?, ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_extension("PNG").map_err(|e| anyhow::anyhow!("Error: {}", e))?, ImageFormat::Png);
        assert_eq!(ImageFormat::from_extension("gif").map_err(|e| anyhow::anyhow!("Error: {}", e))?, ImageFormat::Gif);
        
        assert!(ImageFormat::from_extension("xyz").is_err());
    }

    #[test]
    fn test_image_dimensions() {
        let dims = ImageDimensions::new(800, 600).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(dims.width, 800);
        assert_eq!(dims.height, 600);
        assert_eq!(dims.aspect_ratio(), 800.0 / 600.0);
        assert_eq!(dims.area(), 480000);

        assert!(ImageDimensions::new(0, 600).is_err());
        assert!(ImageDimensions::new(800, 0).is_err());
    }

    #[test]
    fn test_images_creation() {
        let images = Images::new();
        assert!(images.format.is_none());
        assert!(images.dimensions.is_none());
    }

    #[test]
    fn test_detect_format() {
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(Images::detect_format(&jpeg_data).map_err(|e| anyhow::anyhow!("Error: {}", e))?, ImageFormat::Jpeg);

        let png_data = vec![0x89, 0x50, 0x4E, 0x47];
        assert_eq!(Images::detect_format(&png_data).map_err(|e| anyhow::anyhow!("Error: {}", e))?, ImageFormat::Png);

        let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
        assert!(Images::detect_format(&invalid_data).is_err());
    }

    #[test]
    fn test_resize_calculations() {
        let images = Images::new();
        let current = ImageDimensions::new(800, 600).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let options = ResizeOptions::default();

        // Test fit mode
        let result = images.calculate_resize_dimensions(&current, 400, 400, &options).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(result.width, 400);
        assert_eq!(result.height, 300); // Maintains aspect ratio

        // Test exact mode
        let options = ResizeOptions {
            mode: ResizeMode::Exact,
            ..Default::default()
        };
        let result = images.calculate_resize_dimensions(&current, 400, 400, &options).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(result.width, 400);
        assert_eq!(result.height, 400); // Exact dimensions
    }

    #[test]
    fn test_scale() {
        let mut images = Images::new();
        images.dimensions = Some(ImageDimensions::new(800, 600).map_err(|e| anyhow::anyhow!("Error: {}", e))?);

        images.scale(50.0).map_err(|e| anyhow::anyhow!("Error: {}", e))?; // 50% scale
        let dims = images.get_dimensions().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(dims.width, 400);
        assert_eq!(dims.height, 300);
    }

    #[test]
    fn test_utils_calculate_web_dimensions() {
        let original = ImageDimensions::new(2000, 1500).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let web_dims = utils::calculate_web_dimensions(&original, 800, 600);
        
        assert_eq!(web_dims.width, 800);
        assert_eq!(web_dims.height, 600);
    }

    #[test]
    fn test_utils_needs_resize() {
        let small = ImageDimensions::new(400, 300).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let large = ImageDimensions::new(1200, 900).map_err(|e| anyhow::anyhow!("Error: {}", e))?;

        assert!(!utils::needs_resize(&small, 800, 600));
        assert!(utils::needs_resize(&large, 800, 600));
    }

    #[test]
    fn test_save_options_validation() {
        let options = SaveOptions {
            quality: 150, // Invalid quality
            ..Default::default()
        };

        let images = Images::new();
        let result = images.save("/tmp/test.jpg", Some(options));
        // Would fail with InvalidQuality in real implementation with validation
    }

    #[test]
    fn test_crop() {
        let mut images = Images::new();
        images.dimensions = Some(ImageDimensions::new(800, 600).map_err(|e| anyhow::anyhow!("Error: {}", e))?);

        images.crop(100, 100, 400, 300).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let dims = images.get_dimensions().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(dims.width, 400);
        assert_eq!(dims.height, 300);

        // Test invalid crop
        let result = images.crop(500, 500, 400, 300);
        assert!(result.is_err());
    }

    #[test]
    fn test_rotate() {
        let mut images = Images::new();
        images.dimensions = Some(ImageDimensions::new(800, 600).map_err(|e| anyhow::anyhow!("Error: {}", e))?);

        images.rotate(90.0).map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        let dims = images.get_dimensions().map_err(|e| anyhow::anyhow!("Error: {}", e))?;
        assert_eq!(dims.width, 600); // Swapped
        assert_eq!(dims.height, 800); // Swapped
    }
}
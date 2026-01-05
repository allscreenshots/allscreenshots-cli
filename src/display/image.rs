use crate::error::{CliError, CliResult};
use image::DynamicImage;
use std::io::Cursor;
use viuer::{print_from_file, Config as ViuerConfig};

/// Terminal image display using viuer
pub struct TerminalImage {
    config: ViuerConfig,
}

impl Default for TerminalImage {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalImage {
    pub fn new() -> Self {
        Self {
            config: ViuerConfig {
                absolute_offset: false,
                x: 0,
                y: 0,
                restore_cursor: false,
                width: Some(80),
                height: Some(24),
                truecolor: true,
                use_kitty: true,
                use_iterm: true,
                ..Default::default()
            },
        }
    }

    /// Create with custom dimensions
    pub fn with_size(width: u32, height: u32) -> Self {
        let mut display = Self::new();
        display.config.width = Some(width);
        display.config.height = Some(height);
        display
    }

    /// Display image from raw bytes
    pub fn display_bytes(&self, image_bytes: &[u8]) -> CliResult<()> {
        let img = image::load_from_memory(image_bytes)
            .map_err(|e| CliError::DisplayError(format!("Failed to decode image: {}", e)))?;

        self.display_image(&img)
    }

    /// Display a DynamicImage
    pub fn display_image(&self, img: &DynamicImage) -> CliResult<()> {
        viuer::print(img, &self.config)
            .map_err(|e| CliError::DisplayError(format!("Failed to display image: {}", e)))?;

        Ok(())
    }

    /// Display image from file path
    pub fn display_file(&self, path: &std::path::Path) -> CliResult<()> {
        print_from_file(path, &self.config)
            .map_err(|e| CliError::DisplayError(format!("Failed to display image: {}", e)))?;

        Ok(())
    }

    /// Get image dimensions from bytes
    pub fn get_dimensions(image_bytes: &[u8]) -> CliResult<(u32, u32)> {
        let reader = image::ImageReader::new(Cursor::new(image_bytes))
            .with_guessed_format()
            .map_err(|e| CliError::DisplayError(format!("Failed to read image format: {}", e)))?;

        let dims = reader
            .into_dimensions()
            .map_err(|e| CliError::DisplayError(format!("Failed to get dimensions: {}", e)))?;

        Ok(dims)
    }

    /// Detect the best display protocol for the current terminal
    pub fn detect_protocol() -> &'static str {
        // Check for Kitty
        if std::env::var("TERM").map(|t| t.contains("kitty")).unwrap_or(false) {
            return "kitty";
        }

        // Check for iTerm2
        if std::env::var("TERM_PROGRAM")
            .map(|t| t == "iTerm.app")
            .unwrap_or(false)
        {
            return "iterm";
        }

        // Check for WezTerm (supports iTerm protocol)
        if std::env::var("TERM_PROGRAM")
            .map(|t| t == "WezTerm")
            .unwrap_or(false)
        {
            return "iterm";
        }

        // Fallback to block characters
        "blocks"
    }

    /// Print info about the detected terminal protocol
    pub fn print_protocol_info() {
        let protocol = Self::detect_protocol();
        println!(
            "Terminal display protocol: {} ({})",
            protocol,
            match protocol {
                "kitty" => "high quality graphics",
                "iterm" => "inline images",
                "sixel" => "sixel graphics",
                _ => "unicode block characters",
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_protocol() {
        // Should return a valid protocol string
        let protocol = TerminalImage::detect_protocol();
        assert!(["kitty", "iterm", "sixel", "blocks"].contains(&protocol));
    }

    #[test]
    fn test_with_size() {
        let display = TerminalImage::with_size(120, 40);
        assert_eq!(display.config.width, Some(120));
        assert_eq!(display.config.height, Some(40));
    }
}

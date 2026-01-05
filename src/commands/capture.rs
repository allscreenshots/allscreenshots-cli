use crate::display::{create_spinner, TerminalImage};
use crate::error::{CliError, CliResult};
use crate::utils::{auto_filename, normalize_url, save_to_file};
use allscreenshots_sdk::{AllscreenshotsClient, ImageFormat, ScreenshotRequest, WaitUntil, BlockLevel};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct CaptureArgs {
    /// URL to capture
    #[arg(required = true)]
    pub url: String,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Device preset (e.g., "Desktop HD", "iPhone 14")
    #[arg(short, long)]
    pub device: Option<String>,

    /// Viewport width in pixels
    #[arg(long)]
    pub width: Option<u32>,

    /// Viewport height in pixels
    #[arg(long)]
    pub height: Option<u32>,

    /// Image format: png, jpeg, webp, pdf
    #[arg(long, default_value = "png")]
    pub format: String,

    /// Capture the full page
    #[arg(long)]
    pub full_page: bool,

    /// Image quality (1-100, for jpeg/webp)
    #[arg(long)]
    pub quality: Option<i32>,

    /// Delay before capture in milliseconds
    #[arg(long)]
    pub delay: Option<i32>,

    /// CSS selector to wait for before capture
    #[arg(long)]
    pub wait_for: Option<String>,

    /// Wait until: load, domcontentloaded, networkidle, commit
    #[arg(long)]
    pub wait_until: Option<String>,

    /// Enable dark mode
    #[arg(long)]
    pub dark_mode: bool,

    /// Block advertisements
    #[arg(long)]
    pub block_ads: bool,

    /// Block cookie banners
    #[arg(long)]
    pub block_cookies: bool,

    /// Block level: none, light, normal, pro, pro_plus, ultimate
    #[arg(long)]
    pub block_level: Option<String>,

    /// CSS selector to capture specific element
    #[arg(long)]
    pub selector: Option<String>,

    /// Custom CSS to inject
    #[arg(long)]
    pub custom_css: Option<String>,

    /// Display image in terminal
    #[arg(long)]
    pub display: bool,

    /// Don't display image in terminal
    #[arg(long)]
    pub no_display: bool,

    /// Copy image to clipboard
    #[arg(long)]
    pub clipboard: bool,
}

impl CaptureArgs {
    /// Check if we should display the image
    pub fn should_display(&self) -> bool {
        if self.no_display {
            return false;
        }
        self.display || self.output.is_none()
    }
}

/// Execute the capture command
pub async fn execute(args: CaptureArgs, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;

    let url = normalize_url(&args.url)?;

    // Build the request
    let mut builder = ScreenshotRequest::builder().url(&url);

    if let Some(ref device) = args.device {
        builder = builder.device(device);
    }

    // Apply custom viewport if width or height specified
    if args.width.is_some() || args.height.is_some() {
        use allscreenshots_sdk::ViewportConfig;
        let mut viewport = ViewportConfig::default();
        if let Some(width) = args.width {
            viewport.width = Some(width as i32);
        }
        if let Some(height) = args.height {
            viewport.height = Some(height as i32);
        }
        builder = builder.viewport(viewport);
    }

    let format = parse_format(&args.format)?;
    builder = builder.format(format.clone());

    if args.full_page {
        builder = builder.full_page(true);
    }

    if let Some(quality) = args.quality {
        builder = builder.quality(quality);
    }

    if let Some(delay) = args.delay {
        builder = builder.delay(delay);
    }

    if let Some(ref wait_for) = args.wait_for {
        builder = builder.wait_for(wait_for);
    }

    if let Some(ref wait_until) = args.wait_until {
        let wait = parse_wait_until(wait_until)?;
        builder = builder.wait_until(wait);
    }

    if args.dark_mode {
        builder = builder.dark_mode(true);
    }

    if args.block_ads {
        builder = builder.block_ads(true);
    }

    if args.block_cookies {
        builder = builder.block_cookie_banners(true);
    }

    if let Some(ref level) = args.block_level {
        let block_level = parse_block_level(level)?;
        builder = builder.block_level(block_level);
    }

    if let Some(ref selector) = args.selector {
        builder = builder.selector(selector);
    }

    if let Some(ref css) = args.custom_css {
        builder = builder.custom_css(css);
    }

    let request = builder.build().map_err(|e| CliError::Other(e.to_string()))?;

    // Create client and capture
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    let spinner = create_spinner(&format!("Capturing {}...", url));

    let image_bytes = client.screenshot(&request).await.map_err(CliError::Sdk)?;

    spinner.finish_and_clear();

    // Get image dimensions
    let dims = TerminalImage::get_dimensions(&image_bytes).ok();
    let size = image_bytes.len();

    // Save to file if output specified
    let output_path = if let Some(ref output) = args.output {
        save_to_file(output, &image_bytes)?;
        Some(output.clone())
    } else {
        None
    };

    // Display in terminal
    if args.should_display() {
        println!();
        let display = TerminalImage::new();
        display.display_bytes(&image_bytes)?;
        println!();
    }

    // Copy to clipboard
    if args.clipboard {
        copy_to_clipboard(&image_bytes)?;
        println!("{}", "Copied to clipboard!".green());
    }

    // Print summary
    print_summary(&url, dims, size, output_path.as_ref());

    Ok(())
}

/// Quick capture for default command (allscreenshots <URL>)
pub async fn quick_capture(
    url: &str,
    api_key: Option<&str>,
    output: Option<&str>,
    device: Option<&str>,
    full_page: bool,
    should_display: bool,
) -> CliResult<()> {
    let api_key = api_key.map(String::from).ok_or(CliError::NoApiKey)?;
    let url = normalize_url(url)?;

    // Build the request
    let mut builder = ScreenshotRequest::builder().url(&url);

    if let Some(device) = device {
        builder = builder.device(device);
    }

    if full_page {
        builder = builder.full_page(true);
    }

    let request = builder.build().map_err(|e| CliError::Other(e.to_string()))?;

    // Create client and capture
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    let spinner = create_spinner(&format!("Capturing {}...", url));
    let image_bytes = client.screenshot(&request).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    // Get image dimensions
    let dims = TerminalImage::get_dimensions(&image_bytes).ok();
    let size = image_bytes.len();

    // Save to file if output specified
    let output_path = if let Some(output) = output {
        let path = PathBuf::from(output);
        save_to_file(&path, &image_bytes)?;
        Some(path)
    } else {
        None
    };

    // Display in terminal
    if should_display {
        println!();
        let display = TerminalImage::new();
        display.display_bytes(&image_bytes)?;
        println!();
    }

    // Print summary
    print_summary(&url, dims, size, output_path.as_ref());

    Ok(())
}

fn parse_format(s: &str) -> CliResult<ImageFormat> {
    match s.to_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::Webp),
        "pdf" => Ok(ImageFormat::Pdf),
        _ => Err(CliError::Other(format!(
            "Invalid format '{}'. Use: png, jpeg, webp, or pdf",
            s
        ))),
    }
}

fn parse_wait_until(s: &str) -> CliResult<WaitUntil> {
    match s.to_lowercase().as_str() {
        "load" => Ok(WaitUntil::Load),
        "domcontentloaded" => Ok(WaitUntil::DomContentLoaded),
        "networkidle" => Ok(WaitUntil::NetworkIdle),
        "commit" => Ok(WaitUntil::Commit),
        _ => Err(CliError::Other(format!(
            "Invalid wait_until '{}'. Use: load, domcontentloaded, networkidle, or commit",
            s
        ))),
    }
}

fn parse_block_level(s: &str) -> CliResult<BlockLevel> {
    match s.to_lowercase().as_str() {
        "none" => Ok(BlockLevel::None),
        "light" => Ok(BlockLevel::Light),
        "normal" => Ok(BlockLevel::Normal),
        "pro" => Ok(BlockLevel::Pro),
        "pro_plus" | "proplus" => Ok(BlockLevel::ProPlus),
        "ultimate" => Ok(BlockLevel::Ultimate),
        _ => Err(CliError::Other(format!(
            "Invalid block_level '{}'. Use: none, light, normal, pro, pro_plus, or ultimate",
            s
        ))),
    }
}

fn copy_to_clipboard(image_bytes: &[u8]) -> CliResult<()> {
    use arboard::{Clipboard, ImageData};
    use image::GenericImageView;

    let img = image::load_from_memory(image_bytes)
        .map_err(|e| CliError::ClipboardError(format!("Failed to decode image: {}", e)))?;

    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    let mut clipboard = Clipboard::new()
        .map_err(|e| CliError::ClipboardError(format!("Failed to access clipboard: {}", e)))?;

    let img_data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: rgba.into_raw().into(),
    };

    clipboard
        .set_image(img_data)
        .map_err(|e| CliError::ClipboardError(format!("Failed to copy to clipboard: {}", e)))?;

    Ok(())
}

fn print_summary(url: &str, dims: Option<(u32, u32)>, size: usize, output: Option<&PathBuf>) {
    println!("{}", "Screenshot captured!".green().bold());
    println!("  URL: {}", url.dimmed());

    if let Some((w, h)) = dims {
        println!("  Size: {}x{}", w, h);
    }

    println!(
        "  File size: {}",
        crate::utils::format_file_size(size as u64)
    );

    if let Some(path) = output {
        println!("  Saved to: {}", path.display().to_string().cyan());
    }
}

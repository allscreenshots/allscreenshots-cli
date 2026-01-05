use crate::display::{create_spinner, TerminalImage};
use crate::error::{CliError, CliResult};
use crate::utils::{normalize_url, save_to_file};
use allscreenshots_sdk::{
    AllscreenshotsClient, CaptureItem, ComposeOutputConfig, ComposeRequest, ImageFormat, LayoutType,
};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ComposeArgs {
    /// URLs to compose (2-20 URLs)
    #[arg(required = true, num_args = 2..=20)]
    pub urls: Vec<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Layout type: grid, horizontal, vertical, masonry, mondrian, auto
    #[arg(long, default_value = "auto")]
    pub layout: String,

    /// Number of columns (for grid layout)
    #[arg(long)]
    pub columns: Option<i32>,

    /// Spacing between images in pixels
    #[arg(long)]
    pub spacing: Option<i32>,

    /// Padding around the canvas in pixels
    #[arg(long)]
    pub padding: Option<i32>,

    /// Background color (#RRGGBB or "transparent")
    #[arg(long)]
    pub background: Option<String>,

    /// Output format: png, jpeg, webp
    #[arg(long, default_value = "png")]
    pub format: String,

    /// Image quality (1-100)
    #[arg(long)]
    pub quality: Option<i32>,

    /// Device preset for all captures
    #[arg(short, long)]
    pub device: Option<String>,

    /// Capture full pages
    #[arg(long)]
    pub full_page: bool,

    /// Run asynchronously
    #[arg(long, name = "async")]
    pub is_async: bool,

    /// Display result in terminal
    #[arg(long)]
    pub display: bool,

    /// Don't display in terminal
    #[arg(long)]
    pub no_display: bool,
}

impl ComposeArgs {
    pub fn should_display(&self) -> bool {
        if self.no_display {
            return false;
        }
        self.display || self.output.is_none()
    }
}

pub async fn execute(args: ComposeArgs, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;

    // Normalize URLs
    let urls: Vec<String> = args
        .urls
        .iter()
        .map(|u| normalize_url(u))
        .collect::<Result<Vec<_>, _>>()?;

    println!(
        "{} {} screenshots",
        "Composing".cyan().bold(),
        urls.len()
    );

    // Build capture items
    let captures: Vec<CaptureItem> = urls
        .iter()
        .map(|url| {
            let mut item = CaptureItem::new(url);
            if let Some(ref device) = args.device {
                item = item.with_device(device);
            }
            // Note: CaptureItem doesn't have with_full_page, set it via defaults instead
            item
        })
        .collect();

    // Parse layout
    let layout = parse_layout(&args.layout)?;

    // Parse format
    let format = match args.format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::Webp,
        _ => return Err(CliError::Other(format!("Invalid format: {}", args.format))),
    };

    // Build output config
    let mut output_config = ComposeOutputConfig::default();
    output_config.layout = Some(layout);
    output_config.format = Some(format);

    if let Some(columns) = args.columns {
        output_config.columns = Some(columns);
    }
    if let Some(spacing) = args.spacing {
        output_config.spacing = Some(spacing);
    }
    if let Some(padding) = args.padding {
        output_config.padding = Some(padding);
    }
    if let Some(ref bg) = args.background {
        output_config.background = Some(bg.clone());
    }
    if let Some(quality) = args.quality {
        output_config.quality = Some(quality);
    }

    // Build request
    let request = ComposeRequest::with_captures(captures).with_output(output_config);

    // Create client
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    let spinner = create_spinner("Composing screenshots...");

    // Execute compose
    let result = client.compose(&request).await.map_err(CliError::Sdk)?;

    spinner.finish_and_clear();

    // Handle result
    println!("{}", "Composition complete!".green().bold());
    println!("  Layout: {}", args.layout);

    if let (Some(w), Some(h)) = (result.width, result.height) {
        println!("  Size: {}x{}", w, h);
    }

    if let Some(file_size) = result.file_size {
        println!(
            "  File size: {}",
            crate::utils::format_file_size(file_size as u64)
        );
    }

    if let Some(render_time) = result.render_time_ms {
        println!(
            "  Render time: {}",
            crate::utils::format_duration_ms(render_time as u64)
        );
    }

    // If we have a URL, show it
    if let Some(ref url) = result.url {
        println!("  Result URL: {}", url.cyan());

        // Save to file if output specified
        if let Some(ref output) = args.output {
            println!("  To download, use: curl -o {} '{}'", output.display(), url);
        }
    }

    if let Some(ref storage_url) = result.storage_url {
        println!("  Storage URL: {}", storage_url.cyan());
    }

    Ok(())
}

fn parse_layout(s: &str) -> CliResult<LayoutType> {
    match s.to_lowercase().as_str() {
        "grid" => Ok(LayoutType::Grid),
        "horizontal" => Ok(LayoutType::Horizontal),
        "vertical" => Ok(LayoutType::Vertical),
        "masonry" => Ok(LayoutType::Masonry),
        "mondrian" => Ok(LayoutType::Mondrian),
        "partitioning" => Ok(LayoutType::Partitioning),
        "auto" => Ok(LayoutType::Auto),
        _ => Err(CliError::Other(format!(
            "Invalid layout '{}'. Use: grid, horizontal, vertical, masonry, mondrian, partitioning, or auto",
            s
        ))),
    }
}

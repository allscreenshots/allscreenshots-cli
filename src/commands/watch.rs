use crate::display::{create_spinner, TerminalImage};
use crate::error::{CliError, CliResult};
use crate::utils::{auto_filename, normalize_url, save_to_file};
use allscreenshots_sdk::{AllscreenshotsClient, ImageFormat, ScreenshotRequest};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Args, Debug)]
pub struct WatchArgs {
    /// URL to watch
    #[arg(required = true)]
    pub url: String,

    /// Interval between captures (e.g., "5s", "1m", "30s")
    #[arg(short, long, default_value = "5s")]
    pub interval: String,

    /// Output directory for saved screenshots
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,

    /// Device preset
    #[arg(short, long)]
    pub device: Option<String>,

    /// Image format
    #[arg(long, default_value = "png")]
    pub format: String,

    /// Capture full page
    #[arg(long)]
    pub full_page: bool,

    /// Maximum number of captures (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_captures: u32,

    /// Don't display in terminal
    #[arg(long)]
    pub no_display: bool,
}

pub async fn execute(args: WatchArgs, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;
    let url = normalize_url(&args.url)?;

    // Parse interval
    let interval = parse_duration(&args.interval)?;

    // Parse format
    let format = match args.format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::Webp,
        _ => return Err(CliError::Other(format!("Invalid format: {}", args.format))),
    };

    println!("{}", "Watch Mode".bold().cyan());
    println!("  URL: {}", url);
    println!(
        "  Interval: {}",
        humantime::format_duration(interval).to_string()
    );
    if let Some(ref dir) = args.output_dir {
        println!("  Output: {}", dir.display());
    }
    if args.max_captures > 0 {
        println!("  Max captures: {}", args.max_captures);
    }
    println!();
    println!("{}", "Press Ctrl+C to stop".dimmed());
    println!();

    // Create client
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    // Build request template
    let mut builder = ScreenshotRequest::builder()
        .url(&url)
        .format(format);

    if let Some(ref device) = args.device {
        builder = builder.device(device);
    }

    if args.full_page {
        builder = builder.full_page(true);
    }

    let request = builder.build().map_err(|e| CliError::Other(e.to_string()))?;

    // Ensure output directory exists
    if let Some(ref dir) = args.output_dir {
        crate::utils::ensure_dir(dir)?;
    }

    let mut capture_count = 0u32;

    loop {
        capture_count += 1;

        let spinner = create_spinner(&format!("Capture #{}: {}...", capture_count, url));

        match client.screenshot(&request).await {
            Ok(image_bytes) => {
                spinner.finish_and_clear();

                let size = image_bytes.len();
                let dims = TerminalImage::get_dimensions(&image_bytes).ok();

                // Save to file if output directory specified
                if let Some(ref dir) = args.output_dir {
                    let filename = auto_filename(&url, &args.format);
                    let path = dir.join(&filename);
                    save_to_file(&path, &image_bytes)?;
                    print!("  {} Saved: {} ", "✓".green(), filename);
                } else {
                    print!("  {} Captured ", "✓".green());
                }

                // Print size info
                if let Some((w, h)) = dims {
                    print!("({}x{}, {}) ", w, h, crate::utils::format_file_size(size as u64));
                } else {
                    print!("({}) ", crate::utils::format_file_size(size as u64));
                }
                println!();

                // Display in terminal
                if !args.no_display {
                    let display = TerminalImage::with_size(60, 20);
                    let _ = display.display_bytes(&image_bytes);
                    println!();
                }
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!("  {} Capture failed: {}", "✗".red(), e);
            }
        }

        // Check max captures
        if args.max_captures > 0 && capture_count >= args.max_captures {
            println!("\n{} Maximum captures ({}) reached", "✓".green(), args.max_captures);
            break;
        }

        // Wait for next interval
        let wait_spinner = create_spinner(&format!(
            "Waiting {}...",
            humantime::format_duration(interval)
        ));
        sleep(interval).await;
        wait_spinner.finish_and_clear();
    }

    Ok(())
}

fn parse_duration(s: &str) -> CliResult<Duration> {
    // Try to parse as humantime format (e.g., "5s", "1m", "30s")
    humantime::parse_duration(s).map_err(|_| {
        CliError::Other(format!(
            "Invalid duration '{}'. Examples: 5s, 30s, 1m, 5m",
            s
        ))
    })
}

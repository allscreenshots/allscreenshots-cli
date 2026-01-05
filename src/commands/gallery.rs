use crate::display::{create_spinner, TerminalImage};
use crate::error::{CliError, CliResult};
use allscreenshots_sdk::{AllscreenshotsClient, JobStatus};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct GalleryArgs {
    /// Directory containing images to display
    #[arg(long)]
    pub dir: Option<PathBuf>,

    /// Maximum number of images to show
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Thumbnail size: small, medium
    #[arg(long, default_value = "small")]
    pub size: String,
}

pub async fn execute(args: GalleryArgs, api_key: Option<String>) -> CliResult<()> {
    let (width, height) = match args.size.as_str() {
        "medium" => (60, 15),
        _ => (40, 10), // small
    };

    if let Some(ref dir) = args.dir {
        display_local_gallery(dir, args.limit, width, height)
    } else {
        display_api_gallery(api_key, args.limit, width, height).await
    }
}

/// Display images from a local directory
fn display_local_gallery(dir: &PathBuf, limit: usize, width: u32, height: u32) -> CliResult<()> {
    if !dir.exists() {
        return Err(CliError::Other(format!(
            "Directory not found: {}",
            dir.display()
        )));
    }

    println!("{}", "Gallery".bold().underline());
    println!("  Source: {}", dir.display().to_string().cyan());
    println!();

    // Find image files
    let mut images: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| CliError::Other(format!("Failed to read directory: {}", e)))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    matches!(
                        ext.to_lowercase().as_str(),
                        "png" | "jpg" | "jpeg" | "webp" | "gif"
                    )
                })
                .unwrap_or(false)
        })
        .collect();

    if images.is_empty() {
        println!("{}", "No images found in directory.".dimmed());
        return Ok(());
    }

    // Sort by modification time (newest first)
    images.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    let display = TerminalImage::with_size(width, height);

    for path in images.iter().take(limit) {
        // Display thumbnail
        if let Err(e) = display.display_file(path) {
            eprintln!("  {} Failed to display: {}", "!".yellow(), e);
            continue;
        }

        // Print filename
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!("  {}", filename.dimmed());
        println!();
    }

    let shown = images.len().min(limit);
    let total = images.len();
    if total > shown {
        println!(
            "{}",
            format!("Showing {} of {} images", shown, total).dimmed()
        );
    }

    Ok(())
}

/// Display images from recent API jobs
async fn display_api_gallery(
    api_key: Option<String>,
    limit: usize,
    width: u32,
    height: u32,
) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    let spinner = create_spinner("Fetching recent screenshots...");
    let jobs = client.list_jobs().await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!("{}", "Gallery".bold().underline());
    println!("  Source: {}", "Recent API jobs".cyan());
    println!();

    // Filter for completed jobs
    let completed_jobs: Vec<_> = jobs
        .into_iter()
        .filter(|job| job.status == JobStatus::Completed && job.result_url.is_some())
        .take(limit)
        .collect();

    if completed_jobs.is_empty() {
        println!("{}", "No completed screenshots found.".dimmed());
        return Ok(());
    }

    let display = TerminalImage::with_size(width, height);

    for job in &completed_jobs {
        // Download the image
        let spinner = create_spinner(&format!("Loading {}...", job.id));
        match client.get_job_result(&job.id).await {
            Ok(bytes) => {
                spinner.finish_and_clear();

                // Display thumbnail
                if let Err(e) = display.display_bytes(&bytes) {
                    eprintln!("  {} Failed to display: {}", "!".yellow(), e);
                    continue;
                }

                // Print job info
                let label = job
                    .url
                    .as_ref()
                    .map(|u| truncate_url(u, 50))
                    .unwrap_or_else(|| job.id.clone());
                println!("  {}", label.dimmed());
                println!();
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!("  {} Failed to load {}: {}", "!".yellow(), job.id, e);
            }
        }
    }

    println!(
        "{}",
        format!("Showing {} screenshots", completed_jobs.len()).dimmed()
    );

    Ok(())
}

/// Truncate a URL for display
fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len - 3])
    }
}

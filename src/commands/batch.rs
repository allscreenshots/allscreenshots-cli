use crate::display::{create_progress_bar, create_spinner};
use crate::error::{CliError, CliResult};
use crate::utils::{batch_output_path, ensure_dir, normalize_url, read_urls_from_file, save_to_file};
use allscreenshots_sdk::{
    AllscreenshotsClient, BulkDefaults, BulkRequest, BulkUrlRequest, ImageFormat,
};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Args, Debug)]
pub struct BatchArgs {
    /// URLs to capture
    #[arg(value_name = "URL")]
    pub urls: Vec<String>,

    /// Read URLs from file (one per line)
    #[arg(short, long, value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Output directory
    #[arg(short, long, default_value = "./screenshots")]
    pub output_dir: PathBuf,

    /// Device preset
    #[arg(short, long)]
    pub device: Option<String>,

    /// Image format
    #[arg(long, default_value = "png")]
    pub format: String,

    /// Capture full page
    #[arg(long)]
    pub full_page: bool,

    /// Show progress bar
    #[arg(long, default_value = "true")]
    pub progress: bool,

    /// Polling interval in seconds
    #[arg(long, default_value = "2")]
    pub poll_interval: u64,
}

pub async fn execute(args: BatchArgs, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;

    // Collect URLs from arguments and/or file
    let mut urls = args.urls.clone();

    if let Some(ref file_path) = args.file {
        let file_urls = read_urls_from_file(file_path)?;
        urls.extend(file_urls);
    }

    if urls.is_empty() {
        return Err(CliError::Other(
            "No URLs provided. Use positional arguments or --file".to_string(),
        ));
    }

    // Normalize URLs
    let urls: Vec<String> = urls
        .into_iter()
        .map(|u| normalize_url(&u))
        .collect::<Result<Vec<_>, _>>()?;

    // Limit check (API limit is 100)
    if urls.len() > 100 {
        return Err(CliError::Other(format!(
            "Too many URLs ({}). Maximum is 100 per batch.",
            urls.len()
        )));
    }

    println!(
        "{} {} URLs",
        "Batch capture:".cyan().bold(),
        urls.len()
    );

    // Ensure output directory exists
    ensure_dir(&args.output_dir)?;

    // Parse format
    let format = match args.format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::Webp,
        "pdf" => ImageFormat::Pdf,
        _ => return Err(CliError::Other(format!("Invalid format: {}", args.format))),
    };

    // Build bulk request with defaults
    let bulk_urls: Vec<BulkUrlRequest> = urls
        .iter()
        .map(|url| BulkUrlRequest::new(url))
        .collect();

    // Create defaults with device, format, full_page
    let mut defaults = BulkDefaults::default();
    defaults.device = args.device.clone();
    defaults.format = Some(format);
    if args.full_page {
        defaults.full_page = Some(true);
    }

    let bulk_request = BulkRequest::new(bulk_urls).with_defaults(defaults);

    // Create client
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    // Start bulk job
    let spinner = create_spinner("Creating batch job...");
    let bulk_job = client
        .create_bulk_job(&bulk_request)
        .await
        .map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!("  Job ID: {}", bulk_job.id.dimmed());

    // Create progress bar
    let progress = create_progress_bar(urls.len() as u64, "Capturing screenshots");

    // Poll for completion
    let poll_interval = Duration::from_secs(args.poll_interval);
    let final_status = loop {
        sleep(poll_interval).await;

        let status = client
            .get_bulk_job(&bulk_job.id)
            .await
            .map_err(CliError::Sdk)?;

        progress.set_position(status.completed_jobs as u64);

        // Exit when job is done (any terminal state)
        match status.status.as_str() {
            "COMPLETED" | "FAILED" | "PARTIAL" => break status,
            _ => continue,
        }
    };

    progress.finish_with_message("Download complete!");

    // Download and save results
    let mut success_count = 0;
    let mut failed_count = 0;

    println!("\n{}", "Saving screenshots...".cyan());

    if let Some(ref jobs) = final_status.jobs {
        for (i, job) in jobs.iter().enumerate() {
            if job.status == "COMPLETED" {
                if job.result_url.is_some() {
                    // Download from job result endpoint
                    match client.get_job_result(&job.id).await {
                        Ok(bytes) => {
                            let output_path =
                                batch_output_path(&args.output_dir, &job.url, i, &args.format);
                            if let Err(e) = save_to_file(&output_path, &bytes) {
                                eprintln!("  {} Failed to save {}: {}", "✗".red(), job.url, e);
                                failed_count += 1;
                            } else {
                                println!("  {} {}", "✓".green(), output_path.display());
                                success_count += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("  {} Failed to download {}: {}", "✗".red(), job.url, e);
                            failed_count += 1;
                        }
                    }
                } else {
                    eprintln!("  {} No result URL for {}", "✗".red(), job.url);
                    failed_count += 1;
                }
            } else {
                let error = job
                    .error_message
                    .as_deref()
                    .unwrap_or("Unknown error");
                eprintln!("  {} {} - {}", "✗".red(), job.url, error);
                failed_count += 1;
            }
        }
    }

    // Summary
    println!("\n{}", "═".repeat(50).dimmed());
    println!("{}", "Batch Summary".bold());
    println!("  Total: {}", urls.len());
    println!("  {} {}", "Successful:".green(), success_count);
    if failed_count > 0 {
        println!("  {} {}", "Failed:".red(), failed_count);
    }
    println!("  Output: {}", args.output_dir.display().to_string().cyan());
    println!("{}", "═".repeat(50).dimmed());

    if failed_count > 0 && success_count == 0 {
        return Err(CliError::Other("All screenshots failed".to_string()));
    }

    Ok(())
}

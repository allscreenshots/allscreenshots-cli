use crate::display::{create_spinner, TerminalImage};
use crate::error::{CliError, CliResult};
use crate::utils::{normalize_url, save_to_file};
use allscreenshots_sdk::{AllscreenshotsClient, ImageFormat, JobStatus, ScreenshotRequest};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Args, Debug)]
pub struct AsyncArgs {
    /// URL to capture
    #[arg(required = true)]
    pub url: String,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Device preset
    #[arg(short, long)]
    pub device: Option<String>,

    /// Image format
    #[arg(long, default_value = "png")]
    pub format: String,

    /// Capture full page
    #[arg(long)]
    pub full_page: bool,

    /// Poll for completion (default: true)
    #[arg(long, default_value = "true")]
    pub poll: bool,

    /// Don't poll, just create the job
    #[arg(long)]
    pub no_poll: bool,

    /// Polling interval in seconds
    #[arg(long, default_value = "2")]
    pub poll_interval: u64,

    /// Display image in terminal
    #[arg(long)]
    pub display: bool,

    /// Don't display image
    #[arg(long)]
    pub no_display: bool,
}

impl AsyncArgs {
    pub fn should_poll(&self) -> bool {
        !self.no_poll && self.poll
    }

    pub fn should_display(&self) -> bool {
        if self.no_display {
            return false;
        }
        self.display || self.output.is_none()
    }
}

pub async fn execute(args: AsyncArgs, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;
    let url = normalize_url(&args.url)?;

    // Build request
    let mut builder = ScreenshotRequest::builder().url(&url);

    if let Some(ref device) = args.device {
        builder = builder.device(device);
    }

    let format = match args.format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        "webp" => ImageFormat::Webp,
        "pdf" => ImageFormat::Pdf,
        _ => return Err(CliError::Other(format!("Invalid format: {}", args.format))),
    };
    builder = builder.format(format);

    if args.full_page {
        builder = builder.full_page(true);
    }

    let request = builder.build().map_err(|e| CliError::Other(e.to_string()))?;

    // Create client
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    // Start async job
    let spinner = create_spinner(&format!("Starting async capture for {}...", url));

    let job = client
        .screenshot_async(&request)
        .await
        .map_err(CliError::Sdk)?;

    spinner.set_message(format!("Job created: {}", job.id));

    if !args.should_poll() {
        spinner.finish_with_message(format!("Job ID: {}", job.id.cyan()));
        println!("\n{}", "Job created successfully!".green());
        println!("  Job ID: {}", job.id.cyan());
        if let Some(ref status_url) = job.status_url {
            println!("  Status URL: {}", status_url.dimmed());
        }
        println!("\nUse `allscreenshots jobs get {}` to check status", job.id);
        return Ok(());
    }

    // Poll for completion
    spinner.set_message("Waiting for job to complete...");

    let poll_interval = Duration::from_secs(args.poll_interval);
    let image_bytes = loop {
        sleep(poll_interval).await;

        let status = client.get_job(&job.id).await.map_err(CliError::Sdk)?;

        match status.status {
            JobStatus::Completed => {
                spinner.set_message("Downloading result...");
                let bytes = client.get_job_result(&job.id).await.map_err(CliError::Sdk)?;
                break bytes;
            }
            JobStatus::Failed => {
                spinner.finish_with_message("Job failed!".red().to_string());
                return Err(CliError::Other(format!(
                    "Screenshot job failed: {}",
                    status.error_message.unwrap_or_else(|| "Unknown error".to_string())
                )));
            }
            JobStatus::Cancelled => {
                spinner.finish_with_message("Job cancelled!".yellow().to_string());
                return Err(CliError::Other("Screenshot job was cancelled".to_string()));
            }
            _ => {
                spinner.set_message(format!("Status: {:?}...", status.status));
            }
        }
    };

    spinner.finish_and_clear();

    // Save to file
    if let Some(ref output) = args.output {
        save_to_file(output, &image_bytes)?;
        println!("{} {}", "Saved to:".green(), output.display());
    }

    // Display in terminal
    if args.should_display() {
        println!();
        let display = TerminalImage::new();
        display.display_bytes(&image_bytes)?;
        println!();
    }

    println!("{}", "Screenshot captured!".green().bold());

    Ok(())
}

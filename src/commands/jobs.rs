use crate::display::{create_spinner, TerminalImage};
use crate::error::{CliError, CliResult};
use crate::utils::save_to_file;
use allscreenshots_sdk::{AllscreenshotsClient, JobStatus};
use clap::{Args, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct JobsCommand {
    #[command(subcommand)]
    pub command: JobsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum JobsSubcommand {
    /// List recent jobs
    List,

    /// Get job status
    Get {
        /// Job ID
        id: String,
    },

    /// Cancel a job
    Cancel {
        /// Job ID
        id: String,
    },

    /// Download job result
    Result {
        /// Job ID
        id: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Display in terminal
        #[arg(long)]
        display: bool,
    },
}

pub async fn execute(cmd: JobsCommand, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    match cmd.command {
        JobsSubcommand::List => list_jobs(&client).await,
        JobsSubcommand::Get { id } => get_job(&client, &id).await,
        JobsSubcommand::Cancel { id } => cancel_job(&client, &id).await,
        JobsSubcommand::Result { id, output, display } => {
            get_result(&client, &id, output, display).await
        }
    }
}

fn status_icon(status: &JobStatus) -> colored::ColoredString {
    match status {
        JobStatus::Completed => "✓".green(),
        JobStatus::Failed => "✗".red(),
        JobStatus::Cancelled => "⊘".yellow(),
        JobStatus::Processing => "⟳".cyan(),
        JobStatus::Queued => "○".dimmed(),
    }
}

fn status_color(status: &JobStatus) -> &'static str {
    match status {
        JobStatus::Completed => "green",
        JobStatus::Failed => "red",
        JobStatus::Cancelled => "yellow",
        JobStatus::Processing => "cyan",
        JobStatus::Queued => "white",
    }
}

async fn list_jobs(client: &AllscreenshotsClient) -> CliResult<()> {
    let spinner = create_spinner("Fetching jobs...");
    let jobs = client.list_jobs().await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    if jobs.is_empty() {
        println!("{}", "No jobs found.".dimmed());
        return Ok(());
    }

    println!("{}", "Recent Jobs".bold().underline());
    println!();

    for job in jobs {
        let icon = status_icon(&job.status);
        let status_str = format!("{:?}", job.status);

        println!(
            "{} {} ({})",
            icon,
            job.id.cyan(),
            status_str.bold()
        );

        if let Some(ref url) = job.url {
            println!("    URL: {}", url.dimmed());
        }

        if let Some(ref created) = job.created_at {
            println!("    Created: {}", created.dimmed());
        }

        if let Some(ref completed) = job.completed_at {
            println!("    Completed: {}", completed.dimmed());
        }

        if job.status == JobStatus::Failed {
            if let Some(ref error) = job.error_message {
                println!("    Error: {}", error.red());
            }
        }

        if let Some(ref result_url) = job.result_url {
            println!("    Result: {}", result_url.dimmed());
        }

        println!();
    }

    Ok(())
}

async fn get_job(client: &AllscreenshotsClient, id: &str) -> CliResult<()> {
    let spinner = create_spinner("Fetching job...");
    let job = client.get_job(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    let color = status_color(&job.status);
    let status_str = format!("{:?}", job.status);

    println!("{}", "Job Details".bold().underline());
    println!();
    println!("  ID: {}", job.id.cyan());
    println!("  Status: {}", status_str.color(color).bold());

    if let Some(ref url) = job.url {
        println!("  URL: {}", url);
    }

    if let Some(ref created) = job.created_at {
        println!("  Created: {}", created);
    }

    if let Some(ref started) = job.started_at {
        println!("  Started: {}", started);
    }

    if let Some(ref completed) = job.completed_at {
        println!("  Completed: {}", completed);
    }

    if let Some(ref expires) = job.expires_at {
        println!("  Expires: {}", expires);
    }

    if let Some(ref result_url) = job.result_url {
        println!("  Result URL: {}", result_url.cyan());
    }

    if let Some(ref error_code) = job.error_code {
        println!("  Error Code: {}", error_code.red());
    }

    if let Some(ref error_msg) = job.error_message {
        println!("  Error: {}", error_msg.red());
    }

    // Hint for getting result
    if job.status == JobStatus::Completed {
        println!(
            "\n{} {}",
            "Tip:".dimmed(),
            format!("Run `allscreenshots jobs result {}` to download", id).dimmed()
        );
    }

    Ok(())
}

async fn cancel_job(client: &AllscreenshotsClient, id: &str) -> CliResult<()> {
    let spinner = create_spinner("Cancelling job...");
    let job = client.cancel_job(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    if job.status == JobStatus::Cancelled {
        println!("{} Job {} cancelled", "✓".green(), id);
    } else {
        println!(
            "{} Job {} is now {:?} (may have completed before cancellation)",
            "!".yellow(),
            id,
            job.status
        );
    }

    Ok(())
}

async fn get_result(
    client: &AllscreenshotsClient,
    id: &str,
    output: Option<PathBuf>,
    display: bool,
) -> CliResult<()> {
    // First check job status
    let spinner = create_spinner("Checking job status...");
    let job = client.get_job(id).await.map_err(CliError::Sdk)?;

    if job.status != JobStatus::Completed {
        spinner.finish_and_clear();
        return Err(CliError::Other(format!(
            "Job is not completed. Current status: {:?}",
            job.status
        )));
    }

    spinner.set_message("Downloading result...");
    let image_bytes = client.get_job_result(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!(
        "{} Downloaded {}",
        "✓".green(),
        crate::utils::format_file_size(image_bytes.len() as u64)
    );

    // Save to file
    if let Some(ref path) = output {
        save_to_file(path, &image_bytes)?;
        println!("  Saved to: {}", path.display().to_string().cyan());
    }

    // Display in terminal
    let should_display = display || output.is_none();
    if should_display {
        println!();
        let terminal_display = TerminalImage::new();
        terminal_display.display_bytes(&image_bytes)?;
        println!();
    }

    Ok(())
}

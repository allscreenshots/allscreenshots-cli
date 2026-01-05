use allscreenshots_sdk::{AllscreenshotsError, ErrorCode};
use colored::Colorize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    Sdk(#[from] AllscreenshotsError),

    #[error("{0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("No API key found")]
    NoApiKey,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read file: {0}")]
    FileReadError(String),

    #[error("Failed to write file: {0}")]
    FileWriteError(String),

    #[error("Image display error: {0}")]
    DisplayError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("{0}")]
    Other(String),
}

impl CliError {
    /// Format the error with user-friendly messages and helpful suggestions
    pub fn format_friendly(&self) -> String {
        match self {
            CliError::NoApiKey => {
                format!(
                    "{}\n\n{}\n  {}\n  {}\n\n{}\n  {}",
                    "No API key found!".red().bold(),
                    "You can provide your API key in one of these ways:".yellow(),
                    "1. Set the ALLSCREENSHOTS_API_KEY environment variable",
                    "2. Run: allscreenshots config add-authtoken <your-key>",
                    "Get your API key at:".dimmed(),
                    "https://dashboard.allscreenshots.com/api-keys".cyan().underline()
                )
            }

            CliError::Sdk(AllscreenshotsError::EnvVarNotSet(_)) => {
                CliError::NoApiKey.format_friendly()
            }

            CliError::Sdk(AllscreenshotsError::ApiError { code, message, status }) => {
                match code {
                    ErrorCode::Unauthorized => {
                        format!(
                            "{}\n\n{}\n  {}\n\n{}\n  {}",
                            "Authentication failed!".red().bold(),
                            "Your API key appears to be invalid.".yellow(),
                            message,
                            "Check your key at:".dimmed(),
                            "https://dashboard.allscreenshots.com/api-keys".cyan().underline()
                        )
                    }
                    ErrorCode::RateLimitExceeded => {
                        format!(
                            "{}\n\n{}\n\n{}\n  {}",
                            "Rate limit exceeded!".red().bold(),
                            "You've made too many requests. Please wait a moment and try again.".yellow(),
                            "Upgrade your plan for higher limits:".dimmed(),
                            "https://dashboard.allscreenshots.com/billing".cyan().underline()
                        )
                    }
                    ErrorCode::ValidationError => {
                        format!(
                            "{}\n\n{}",
                            "Invalid request!".red().bold(),
                            message.yellow()
                        )
                    }
                    ErrorCode::NotFound => {
                        format!(
                            "{}\n\n{}",
                            "Resource not found!".red().bold(),
                            message.yellow()
                        )
                    }
                    _ => {
                        format!(
                            "{} (HTTP {})\n\n{}",
                            "API Error".red().bold(),
                            status,
                            message
                        )
                    }
                }
            }

            CliError::Sdk(AllscreenshotsError::HttpError(e)) => {
                if e.is_timeout() {
                    format!(
                        "{}\n\n{}\n  {}",
                        "Request timed out!".red().bold(),
                        "The server took too long to respond.".yellow(),
                        "Try again or use --timeout to increase the limit".dimmed()
                    )
                } else if e.is_connect() {
                    format!(
                        "{}\n\n{}\n  {}",
                        "Connection failed!".red().bold(),
                        "Could not connect to the AllScreenshots API.".yellow(),
                        "Check your internet connection and try again.".dimmed()
                    )
                } else {
                    format!(
                        "{}\n\n{}",
                        "Network error!".red().bold(),
                        e.to_string().yellow()
                    )
                }
            }

            CliError::Sdk(AllscreenshotsError::ValidationError(msg)) => {
                format!(
                    "{}\n\n{}",
                    "Invalid input!".red().bold(),
                    msg.yellow()
                )
            }

            CliError::Sdk(AllscreenshotsError::RetriesExhausted(msg)) => {
                format!(
                    "{}\n\n{}\n  {}",
                    "Request failed after retries!".red().bold(),
                    "All retry attempts failed.".yellow(),
                    format!("Last error: {}", msg).dimmed()
                )
            }

            CliError::Sdk(AllscreenshotsError::Timeout) => {
                format!(
                    "{}\n\n{}\n  {}",
                    "Request timed out!".red().bold(),
                    "The request took too long.".yellow(),
                    "Try increasing the timeout with --timeout".dimmed()
                )
            }

            CliError::InvalidUrl(url) => {
                format!(
                    "{}\n\n{}\n  {}",
                    "Invalid URL!".red().bold(),
                    format!("'{}' is not a valid URL.", url).yellow(),
                    "URLs should start with http:// or https://".dimmed()
                )
            }

            CliError::FileNotFound(path) => {
                format!(
                    "{}\n\n{}",
                    "File not found!".red().bold(),
                    format!("Could not find: {}", path).yellow()
                )
            }

            CliError::FileReadError(msg) => {
                format!(
                    "{}\n\n{}",
                    "Failed to read file!".red().bold(),
                    msg.yellow()
                )
            }

            CliError::FileWriteError(msg) => {
                format!(
                    "{}\n\n{}",
                    "Failed to write file!".red().bold(),
                    msg.yellow()
                )
            }

            CliError::DisplayError(msg) => {
                format!(
                    "{}\n\n{}\n  {}",
                    "Failed to display image!".red().bold(),
                    msg.yellow(),
                    "Try using --no-display to skip terminal display".dimmed()
                )
            }

            CliError::ClipboardError(msg) => {
                format!(
                    "{}\n\n{}",
                    "Clipboard error!".red().bold(),
                    msg.yellow()
                )
            }

            CliError::Config(e) => {
                format!(
                    "{}\n\n{}",
                    "Configuration error!".red().bold(),
                    e.to_string().yellow()
                )
            }

            CliError::Other(msg) => {
                format!(
                    "{}\n\n{}",
                    "Error!".red().bold(),
                    msg.yellow()
                )
            }

            CliError::Sdk(e) => {
                format!(
                    "{}\n\n{}",
                    "SDK Error!".red().bold(),
                    e.to_string().yellow()
                )
            }
        }
    }

    /// Print the error with friendly formatting
    pub fn print_friendly(&self) {
        eprintln!("\n{}\n", self.format_friendly());
    }
}

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

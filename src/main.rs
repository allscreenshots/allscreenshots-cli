use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod commands;
mod config;
mod display;
mod error;
mod utils;

use config::Config;
use error::{CliError, CliResult};

#[derive(Parser)]
#[command(
    name = "allscreenshots",
    author = "Allscreenshots <support@allscreenshots.com>",
    version,
    about = "Capture website screenshots from the command line",
    long_about = "A modern CLI tool for capturing website screenshots using the AllScreenshots API.\n\n\
                  Get your API key at: https://dashboard.allscreenshots.com/api-keys",
    after_help = "EXAMPLES:\n\
                  \n\
                  Quick screenshot (displays in terminal):\n    \
                    allscreenshots https://www.google.com\n\
                  \n\
                  Save to file:\n    \
                    allscreenshots https://github.com -o github.png\n\
                  \n\
                  Mobile screenshot:\n    \
                    allscreenshots https://example.com --device \"iPhone 14\" --full-page\n\
                  \n\
                  Batch capture from file:\n    \
                    allscreenshots batch -f urls.txt -o ./screenshots/\n\
                  \n\
                  Check usage:\n    \
                    allscreenshots usage\n\
                  \n\
                  Set up authentication:\n    \
                    allscreenshots config add-authtoken <your-api-key>"
)]
struct Cli {
    /// URL to capture (shorthand for `allscreenshots capture <URL>`)
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// API key (overrides ALLSCREENSHOTS_API_KEY and config file)
    #[arg(short = 'k', long, global = true, env = "ALLSCREENSHOTS_API_KEY")]
    api_key: Option<String>,

    /// Output file path
    #[arg(short, long, global = true)]
    output: Option<PathBuf>,

    /// Device preset (e.g., "Desktop HD", "iPhone 14")
    #[arg(short, long, global = true)]
    device: Option<String>,

    /// Capture full page
    #[arg(long, global = true)]
    full_page: bool,

    /// Show image in terminal
    #[arg(long, global = true)]
    display: bool,

    /// Don't show image in terminal
    #[arg(long, global = true)]
    no_display: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Take a synchronous screenshot
    Capture(commands::capture::CaptureArgs),

    /// Take an async screenshot with job tracking
    Async(commands::async_capture::AsyncArgs),

    /// Capture multiple URLs (bulk operation)
    Batch(commands::batch::BatchArgs),

    /// Combine multiple screenshots into one image
    Compose(commands::compose::ComposeArgs),

    /// Manage scheduled screenshots
    Schedule(commands::schedule::ScheduleCommand),

    /// Show API usage and quota
    Usage(commands::usage::UsageArgs),

    /// Manage authentication and settings
    Config(commands::config::ConfigCommand),

    /// List and manage screenshot jobs
    Jobs(commands::jobs::JobsCommand),

    /// Browse screenshots with thumbnails
    Gallery(commands::gallery::GalleryArgs),

    /// Watch mode - re-capture at intervals
    Watch(commands::watch::WatchArgs),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_name = "SHELL")]
        shell: String,

        /// Show installation instructions
        #[arg(long)]
        instructions: bool,
    },

    /// Show available device presets
    Devices,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Disable colors if requested
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Run the CLI
    if let Err(e) = run(cli).await {
        e.print_friendly();
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> CliResult<()> {
    // Load config
    let config = Config::load().unwrap_or_default();

    // Get API key with priority: CLI > env > config
    let api_key = cli
        .api_key
        .or_else(|| std::env::var("ALLSCREENSHOTS_API_KEY").ok())
        .or_else(|| config.auth.api_key.clone());

    // Handle quick capture (allscreenshots <URL>)
    if let Some(ref url) = cli.url {
        let should_display = if cli.no_display {
            false
        } else {
            cli.display || cli.output.is_none()
        };

        return commands::capture::quick_capture(
            url,
            api_key.as_deref(),
            cli.output.as_ref().map(|p| p.to_str().unwrap()),
            cli.device.as_deref(),
            cli.full_page,
            should_display,
        )
        .await;
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Capture(args)) => {
            commands::capture::execute(args, api_key).await
        }
        Some(Commands::Async(args)) => {
            commands::async_capture::execute(args, api_key).await
        }
        Some(Commands::Batch(args)) => {
            commands::batch::execute(args, api_key).await
        }
        Some(Commands::Compose(args)) => {
            commands::compose::execute(args, api_key).await
        }
        Some(Commands::Schedule(cmd)) => {
            commands::schedule::execute(cmd, api_key).await
        }
        Some(Commands::Usage(args)) => {
            commands::usage::execute(args, api_key).await
        }
        Some(Commands::Config(cmd)) => {
            commands::config::execute(cmd).await
        }
        Some(Commands::Jobs(cmd)) => {
            commands::jobs::execute(cmd, api_key).await
        }
        Some(Commands::Gallery(args)) => {
            commands::gallery::execute(args, api_key).await
        }
        Some(Commands::Watch(args)) => {
            commands::watch::execute(args, api_key).await
        }
        Some(Commands::Completions { shell, instructions }) => {
            let shell = commands::completions::parse_shell(&shell)?;
            if instructions {
                commands::completions::print_instructions(shell);
                Ok(())
            } else {
                use clap::CommandFactory;
                let mut cmd = Cli::command();
                commands::completions::generate_completions(shell, &mut cmd)
            }
        }
        Some(Commands::Devices) => {
            print_devices();
            Ok(())
        }
        None => {
            // No URL and no subcommand - show help
            print_welcome();
            Ok(())
        }
    }
}

fn print_welcome() {
    println!();
    println!(
        "{}",
        "  █████╗ ██╗     ██╗     ███████╗ ██████╗██████╗ ███████╗███████╗███╗   ██╗███████╗██╗  ██╗ ██████╗ ████████╗███████╗"
            .cyan()
    );
    println!(
        "{}",
        " ██╔══██╗██║     ██║     ██╔════╝██╔════╝██╔══██╗██╔════╝██╔════╝████╗  ██║██╔════╝██║  ██║██╔═══██╗╚══██╔══╝██╔════╝"
            .cyan()
    );
    println!(
        "{}",
        " ███████║██║     ██║     ███████╗██║     ██████╔╝█████╗  █████╗  ██╔██╗ ██║███████╗███████║██║   ██║   ██║   ███████╗"
            .cyan()
    );
    println!(
        "{}",
        " ██╔══██║██║     ██║     ╚════██║██║     ██╔══██╗██╔══╝  ██╔══╝  ██║╚██╗██║╚════██║██╔══██║██║   ██║   ██║   ╚════██║"
            .cyan()
    );
    println!(
        "{}",
        " ██║  ██║███████╗███████╗███████║╚██████╗██║  ██║███████╗███████╗██║ ╚████║███████║██║  ██║╚██████╔╝   ██║   ███████║"
            .cyan()
    );
    println!(
        "{}",
        " ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝    ╚═╝   ╚══════╝"
            .cyan()
    );
    println!();
    println!(
        "  {}",
        "Capture website screenshots from the command line".dimmed()
    );
    println!();

    // Check for API key
    let config = Config::load().unwrap_or_default();
    let has_key = config.get_api_key().is_some();

    if !has_key {
        println!("  {}", "Getting Started".bold().yellow());
        println!();
        println!(
            "  1. Get your API key at: {}",
            "https://dashboard.allscreenshots.com/api-keys".cyan().underline()
        );
        println!(
            "  2. Set it up: {}",
            "allscreenshots config add-authtoken <your-key>".green()
        );
        println!();
    }

    println!("  {}", "Quick Examples".bold());
    println!();
    println!(
        "  {}  {}",
        "allscreenshots https://google.com".green(),
        "# Capture and display".dimmed()
    );
    println!(
        "  {}  {}",
        "allscreenshots https://github.com -o github.png".green(),
        "# Save to file".dimmed()
    );
    println!(
        "  {}  {}",
        "allscreenshots usage".green(),
        "# Check API usage".dimmed()
    );
    println!();
    println!(
        "  Run {} for all commands",
        "allscreenshots --help".cyan()
    );
    println!();
}

fn print_devices() {
    println!("{}", "Available Device Presets".bold().underline());
    println!();

    println!("{}", "Desktop".cyan().bold());
    for (name, resolution) in utils::device_presets().iter().filter(|(n, _)| {
        n.starts_with("Desktop") || n.starts_with("Laptop")
    }) {
        println!("  {:<25} {}", name, resolution.dimmed());
    }

    println!();
    println!("{}", "Tablet".cyan().bold());
    for (name, resolution) in utils::device_presets().iter().filter(|(n, _)| {
        n.starts_with("Tablet") || n.starts_with("iPad")
    }) {
        println!("  {:<25} {}", name, resolution.dimmed());
    }

    println!();
    println!("{}", "Mobile".cyan().bold());
    for (name, resolution) in utils::device_presets().iter().filter(|(n, _)| {
        n.starts_with("iPhone") || n.starts_with("Android")
    }) {
        println!("  {:<25} {}", name, resolution.dimmed());
    }

    println!();
    println!(
        "{}",
        "Use with: allscreenshots <url> --device \"Device Name\"".dimmed()
    );
}

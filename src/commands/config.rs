use crate::config::Config;
use crate::error::{CliError, CliResult};
use clap::{Args, Subcommand};
use colored::Colorize;

#[derive(Args, Debug)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// Add your API authentication token
    #[command(name = "add-authtoken")]
    AddAuthToken {
        /// Your API key from dashboard.allscreenshots.com
        token: String,
    },

    /// Show current configuration
    Show,

    /// Show config file path
    Path,

    /// Remove stored API key
    #[command(name = "remove-authtoken")]
    RemoveAuthToken,

    /// Set a configuration option
    Set {
        /// Configuration key (e.g., "defaults.device")
        key: String,
        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
}

pub async fn execute(cmd: ConfigCommand) -> CliResult<()> {
    match cmd.command {
        ConfigSubcommand::AddAuthToken { token } => add_auth_token(&token),
        ConfigSubcommand::Show => show_config(),
        ConfigSubcommand::Path => show_path(),
        ConfigSubcommand::RemoveAuthToken => remove_auth_token(),
        ConfigSubcommand::Set { key, value } => set_config(&key, &value),
        ConfigSubcommand::Get { key } => get_config(&key),
    }
}

fn add_auth_token(token: &str) -> CliResult<()> {
    // Validate token format
    if token.is_empty() {
        return Err(CliError::Other("Token cannot be empty".to_string()));
    }

    let mut config = Config::load().map_err(CliError::Config)?;
    config.set_api_key(token.to_string()).map_err(CliError::Config)?;

    println!("{}", "Authentication token saved!".green().bold());
    println!(
        "  Token: {}",
        Config::mask_api_key(token).dimmed()
    );
    println!(
        "\nYou can now use {} to capture screenshots.",
        "allscreenshots <url>".cyan()
    );

    Ok(())
}

fn show_config() -> CliResult<()> {
    let config = Config::load().map_err(CliError::Config)?;

    println!("{}", "Current Configuration".bold().underline());
    println!();

    // Auth
    println!("{}", "[auth]".cyan());
    if let Some(ref key) = config.auth.api_key {
        println!("  api_key = \"{}\"", Config::mask_api_key(key));
    } else {
        let env_key = std::env::var("ALLSCREENSHOTS_API_KEY").ok();
        if let Some(ref key) = env_key {
            println!(
                "  api_key = \"{}\" {}",
                Config::mask_api_key(key),
                "(from env)".dimmed()
            );
        } else {
            println!("  api_key = {}", "(not set)".dimmed());
        }
    }

    // Defaults
    println!("\n{}", "[defaults]".cyan());
    if let Some(ref device) = config.defaults.device {
        println!("  device = \"{}\"", device);
    }
    if let Some(ref format) = config.defaults.format {
        println!("  format = \"{}\"", format);
    }
    if let Some(ref dir) = config.defaults.output_dir {
        println!("  output_dir = \"{}\"", dir);
    }
    if let Some(display) = config.defaults.display {
        println!("  display = {}", display);
    }

    // Display
    println!("\n{}", "[display]".cyan());
    if let Some(ref protocol) = config.display.protocol {
        println!("  protocol = \"{}\"", protocol);
    }
    if let Some(width) = config.display.width {
        println!("  width = {}", width);
    }
    if let Some(height) = config.display.height {
        println!("  height = {}", height);
    }

    println!();

    Ok(())
}

fn show_path() -> CliResult<()> {
    match Config::config_path() {
        Some(path) => {
            println!("{}", path.display());
            if path.exists() {
                println!("{}", "(file exists)".dimmed());
            } else {
                println!("{}", "(file does not exist yet)".dimmed());
            }
        }
        None => {
            return Err(CliError::Other(
                "Could not determine config path".to_string(),
            ));
        }
    }

    Ok(())
}

fn remove_auth_token() -> CliResult<()> {
    let mut config = Config::load().map_err(CliError::Config)?;

    if config.auth.api_key.is_none() {
        println!("{}", "No API key is stored in config.".dimmed());
        return Ok(());
    }

    config.remove_api_key().map_err(CliError::Config)?;

    println!("{}", "Authentication token removed.".yellow());
    println!(
        "\n{}",
        "Note: If ALLSCREENSHOTS_API_KEY environment variable is set, it will still be used."
            .dimmed()
    );

    Ok(())
}

fn set_config(key: &str, value: &str) -> CliResult<()> {
    let mut config = Config::load().map_err(CliError::Config)?;

    match key {
        "defaults.device" => {
            config.defaults.device = Some(value.to_string());
        }
        "defaults.format" => {
            config.defaults.format = Some(value.to_string());
        }
        "defaults.output_dir" => {
            config.defaults.output_dir = Some(value.to_string());
        }
        "defaults.display" => {
            config.defaults.display = Some(value.parse().map_err(|_| {
                CliError::Other("Value must be 'true' or 'false'".to_string())
            })?);
        }
        "display.protocol" => {
            config.display.protocol = Some(value.to_string());
        }
        "display.width" => {
            config.display.width = Some(value.parse().map_err(|_| {
                CliError::Other("Value must be a number".to_string())
            })?);
        }
        "display.height" => {
            config.display.height = Some(value.parse().map_err(|_| {
                CliError::Other("Value must be a number".to_string())
            })?);
        }
        _ => {
            return Err(CliError::Other(format!(
                "Unknown config key: {}. Valid keys: defaults.device, defaults.format, defaults.output_dir, defaults.display, display.protocol, display.width, display.height",
                key
            )));
        }
    }

    config.save().map_err(CliError::Config)?;

    println!("{} {} = \"{}\"", "Set".green(), key, value);

    Ok(())
}

fn get_config(key: &str) -> CliResult<()> {
    let config = Config::load().map_err(CliError::Config)?;

    let value: Option<String> = match key {
        "auth.api_key" => config.auth.api_key.map(|k| Config::mask_api_key(&k)),
        "defaults.device" => config.defaults.device,
        "defaults.format" => config.defaults.format,
        "defaults.output_dir" => config.defaults.output_dir,
        "defaults.display" => config.defaults.display.map(|v| v.to_string()),
        "display.protocol" => config.display.protocol,
        "display.width" => config.display.width.map(|v| v.to_string()),
        "display.height" => config.display.height.map(|v| v.to_string()),
        _ => {
            return Err(CliError::Other(format!("Unknown config key: {}", key)));
        }
    };

    match value {
        Some(v) => println!("{}", v),
        None => println!("{}", "(not set)".dimmed()),
    }

    Ok(())
}

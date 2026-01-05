use crate::display::{create_spinner, UsageGraph};
use crate::error::{CliError, CliResult};
use allscreenshots_sdk::AllscreenshotsClient;
use clap::Args;
use colored::Colorize;

#[derive(Args, Debug)]
pub struct UsageArgs {
    /// Output format: table, graph, json
    #[arg(long, default_value = "graph")]
    pub format: String,

    /// Show quota status only (simpler view)
    #[arg(long)]
    pub quota_only: bool,
}

pub async fn execute(args: UsageArgs, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    if args.quota_only {
        return show_quota(&client).await;
    }

    match args.format.as_str() {
        "json" => show_usage_json(&client).await,
        "table" => show_usage_table(&client).await,
        _ => show_usage_graph(&client).await,
    }
}

async fn show_usage_graph(client: &AllscreenshotsClient) -> CliResult<()> {
    let spinner = create_spinner("Fetching usage data...");
    let usage = client.get_usage().await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    UsageGraph::render_usage_summary(&usage);

    Ok(())
}

async fn show_quota(client: &AllscreenshotsClient) -> CliResult<()> {
    let spinner = create_spinner("Fetching quota...");
    let quota = client.get_quota().await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    UsageGraph::render_quota_status(&quota);

    Ok(())
}

async fn show_usage_table(client: &AllscreenshotsClient) -> CliResult<()> {
    let spinner = create_spinner("Fetching usage data...");
    let usage = client.get_usage().await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!("\n{}", "API Usage".bold().underline());
    println!();

    // Tier info
    println!("{:<20} {}", "Tier:", usage.tier.cyan());

    // Current period (not Option)
    let period = &usage.current_period;
    println!();
    println!("{}", "Current Period".bold());
    println!("{:<20} {}", "  Start:", period.period_start);
    println!("{:<20} {}", "  End:", period.period_end);
    println!("{:<20} {}", "  Screenshots:", period.screenshots_count);
    println!("{:<20} {}", "  Bandwidth:", period.bandwidth_formatted);

    // Quota
    if let Some(ref quota) = usage.quota {
        println!();
        println!("{}", "Quota".bold());
        println!(
            "{:<20} {} / {} ({}% used)",
            "  Screenshots:",
            quota.screenshots.used,
            quota.screenshots.limit,
            quota.screenshots.percent_used
        );
        println!(
            "{:<20} {}",
            "  Remaining:",
            quota.screenshots.remaining.to_string().green()
        );

        println!(
            "{:<20} {} / {} ({}% used)",
            "  Bandwidth:",
            quota.bandwidth.used_formatted,
            quota.bandwidth.limit_formatted,
            quota.bandwidth.percent_used
        );
    }

    // Totals
    if let Some(ref totals) = usage.totals {
        println!();
        println!("{}", "All-Time Totals".bold());
        println!("{:<20} {}", "  Screenshots:", totals.screenshots_count);
        println!("{:<20} {}", "  Bandwidth:", totals.bandwidth_formatted);
    }

    println!();

    Ok(())
}

async fn show_usage_json(client: &AllscreenshotsClient) -> CliResult<()> {
    let usage = client.get_usage().await.map_err(CliError::Sdk)?;

    let json = serde_json::to_string_pretty(&usage)
        .map_err(|e| CliError::Other(format!("Failed to serialize: {}", e)))?;

    println!("{}", json);

    Ok(())
}

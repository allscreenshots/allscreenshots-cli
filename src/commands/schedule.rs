use crate::display::create_spinner;
use crate::error::{CliError, CliResult};
use crate::utils::normalize_url;
use allscreenshots_sdk::{AllscreenshotsClient, CreateScheduleRequest, UpdateScheduleRequest};
use clap::{Args, Subcommand};
use colored::Colorize;

#[derive(Args, Debug)]
pub struct ScheduleCommand {
    #[command(subcommand)]
    pub command: ScheduleSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ScheduleSubcommand {
    /// List all schedules
    List,

    /// Create a new schedule
    Create(CreateScheduleArgs),

    /// Get schedule details
    Get {
        /// Schedule ID
        id: String,
    },

    /// Update a schedule
    Update(UpdateScheduleArgs),

    /// Delete a schedule
    Delete {
        /// Schedule ID
        id: String,
    },

    /// Pause a schedule
    Pause {
        /// Schedule ID
        id: String,
    },

    /// Resume a schedule
    Resume {
        /// Schedule ID
        id: String,
    },

    /// Trigger a schedule immediately
    Trigger {
        /// Schedule ID
        id: String,
    },

    /// View execution history
    History {
        /// Schedule ID
        id: String,

        /// Maximum number of entries
        #[arg(long, default_value = "10")]
        limit: Option<i32>,
    },
}

#[derive(Args, Debug)]
pub struct CreateScheduleArgs {
    /// Schedule name
    #[arg(long)]
    pub name: String,

    /// URL to capture
    #[arg(required = true)]
    pub url: String,

    /// Cron expression (e.g., "0 9 * * *" for daily at 9am)
    #[arg(long)]
    pub cron: String,

    /// Timezone (e.g., "America/New_York")
    #[arg(long)]
    pub timezone: Option<String>,

    /// Device preset
    #[arg(short, long)]
    pub device: Option<String>,

    /// Retention days (1-365)
    #[arg(long)]
    pub retention_days: Option<i32>,

    /// Webhook URL for notifications
    #[arg(long)]
    pub webhook_url: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateScheduleArgs {
    /// Schedule ID
    pub id: String,

    /// New name
    #[arg(long)]
    pub name: Option<String>,

    /// New URL
    #[arg(long)]
    pub url: Option<String>,

    /// New cron expression
    #[arg(long)]
    pub cron: Option<String>,

    /// New timezone
    #[arg(long)]
    pub timezone: Option<String>,

    /// New retention days
    #[arg(long)]
    pub retention_days: Option<i32>,
}

pub async fn execute(cmd: ScheduleCommand, api_key: Option<String>) -> CliResult<()> {
    let api_key = api_key.ok_or(CliError::NoApiKey)?;
    let client = AllscreenshotsClient::new(&api_key).map_err(CliError::Sdk)?;

    match cmd.command {
        ScheduleSubcommand::List => list_schedules(&client).await,
        ScheduleSubcommand::Create(args) => create_schedule(&client, args).await,
        ScheduleSubcommand::Get { id } => get_schedule(&client, &id).await,
        ScheduleSubcommand::Update(args) => update_schedule(&client, args).await,
        ScheduleSubcommand::Delete { id } => delete_schedule(&client, &id).await,
        ScheduleSubcommand::Pause { id } => pause_schedule(&client, &id).await,
        ScheduleSubcommand::Resume { id } => resume_schedule(&client, &id).await,
        ScheduleSubcommand::Trigger { id } => trigger_schedule(&client, &id).await,
        ScheduleSubcommand::History { id, limit } => get_history(&client, &id, limit).await,
    }
}

async fn list_schedules(client: &AllscreenshotsClient) -> CliResult<()> {
    let spinner = create_spinner("Fetching schedules...");
    let schedules = client.list_schedules().await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    if schedules.schedules.is_empty() {
        println!("{}", "No schedules found.".dimmed());
        return Ok(());
    }

    println!("{}", "Schedules".bold().underline());
    println!();

    for schedule in schedules.schedules {
        let status_color = match schedule.status.as_str() {
            "ACTIVE" => "green",
            "PAUSED" => "yellow",
            _ => "white",
        };

        println!(
            "{} {} ({})",
            "•".color(status_color),
            schedule.name.bold(),
            schedule.id.dimmed()
        );
        println!("    URL: {}", schedule.url);
        let tz = schedule.timezone.as_deref().unwrap_or("UTC");
        println!("    Schedule: {} ({})", schedule.schedule, tz);
        if let Some(ref desc) = schedule.schedule_description {
            println!("    Description: {}", desc.dimmed());
        }
        println!("    Status: {}", schedule.status.color(status_color));
        if let Some(ref next) = schedule.next_execution_at {
            println!("    Next run: {}", next.cyan());
        }
        let exec_count = schedule.execution_count.unwrap_or(0);
        let success = schedule.success_count.unwrap_or(0);
        let failed = schedule.failure_count.unwrap_or(0);
        println!(
            "    Executions: {} total ({} success, {} failed)",
            exec_count,
            success.to_string().green(),
            failed.to_string().red()
        );
        println!();
    }

    Ok(())
}

async fn create_schedule(client: &AllscreenshotsClient, args: CreateScheduleArgs) -> CliResult<()> {
    let url = normalize_url(&args.url)?;

    let mut request = CreateScheduleRequest::new(&args.name, &url, &args.cron);

    if let Some(ref tz) = args.timezone {
        request = request.with_timezone(tz);
    }

    if let Some(days) = args.retention_days {
        request = request.with_retention_days(days);
    }

    if let Some(ref webhook) = args.webhook_url {
        request.webhook_url = Some(webhook.clone());
    }

    let spinner = create_spinner("Creating schedule...");
    let schedule = client
        .create_schedule(&request)
        .await
        .map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!("{}", "Schedule created!".green().bold());
    println!("  ID: {}", schedule.id.cyan());
    println!("  Name: {}", schedule.name);
    println!("  URL: {}", schedule.url);
    println!("  Schedule: {}", schedule.schedule);
    if let Some(ref next) = schedule.next_execution_at {
        println!("  Next execution: {}", next.cyan());
    }

    Ok(())
}

async fn get_schedule(client: &AllscreenshotsClient, id: &str) -> CliResult<()> {
    let spinner = create_spinner("Fetching schedule...");
    let schedule = client.get_schedule(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!("{}", "Schedule Details".bold().underline());
    println!();
    println!("  ID: {}", schedule.id.cyan());
    println!("  Name: {}", schedule.name.bold());
    println!("  URL: {}", schedule.url);
    println!("  Schedule: {}", schedule.schedule);
    if let Some(ref desc) = schedule.schedule_description {
        println!("  Description: {}", desc);
    }
    let tz = schedule.timezone.as_deref().unwrap_or("UTC");
    println!("  Timezone: {}", tz);
    println!("  Status: {}", schedule.status);
    if let Some(ref last) = schedule.last_executed_at {
        println!("  Last executed: {}", last);
    }
    if let Some(ref next) = schedule.next_execution_at {
        println!("  Next execution: {}", next.cyan());
    }
    let exec_count = schedule.execution_count.unwrap_or(0);
    let success = schedule.success_count.unwrap_or(0);
    let failed = schedule.failure_count.unwrap_or(0);
    println!(
        "  Executions: {} ({} success, {} failed)",
        exec_count,
        success.to_string().green(),
        failed.to_string().red()
    );
    if let Some(ref created) = schedule.created_at {
        println!("  Created: {}", created.dimmed());
    }

    Ok(())
}

async fn update_schedule(client: &AllscreenshotsClient, args: UpdateScheduleArgs) -> CliResult<()> {
    let mut request = UpdateScheduleRequest::default();

    if let Some(ref name) = args.name {
        request.name = Some(name.clone());
    }
    if let Some(ref url) = args.url {
        request.url = Some(normalize_url(url)?);
    }
    if let Some(ref cron) = args.cron {
        request.schedule = Some(cron.clone());
    }
    if let Some(ref tz) = args.timezone {
        request.timezone = Some(tz.clone());
    }
    if let Some(days) = args.retention_days {
        request.retention_days = Some(days);
    }

    let spinner = create_spinner("Updating schedule...");
    let schedule = client
        .update_schedule(&args.id, &request)
        .await
        .map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!("{}", "Schedule updated!".green().bold());
    println!("  ID: {}", schedule.id);
    println!("  Name: {}", schedule.name);

    Ok(())
}

async fn delete_schedule(client: &AllscreenshotsClient, id: &str) -> CliResult<()> {
    let spinner = create_spinner("Deleting schedule...");
    client.delete_schedule(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!("{} Schedule {} deleted", "✓".green(), id);

    Ok(())
}

async fn pause_schedule(client: &AllscreenshotsClient, id: &str) -> CliResult<()> {
    let spinner = create_spinner("Pausing schedule...");
    let schedule = client.pause_schedule(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!(
        "{} Schedule {} paused",
        "⏸".yellow(),
        schedule.name.bold()
    );

    Ok(())
}

async fn resume_schedule(client: &AllscreenshotsClient, id: &str) -> CliResult<()> {
    let spinner = create_spinner("Resuming schedule...");
    let schedule = client.resume_schedule(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!(
        "{} Schedule {} resumed",
        "▶".green(),
        schedule.name.bold()
    );
    if let Some(ref next) = schedule.next_execution_at {
        println!("  Next execution: {}", next.cyan());
    }

    Ok(())
}

async fn trigger_schedule(client: &AllscreenshotsClient, id: &str) -> CliResult<()> {
    let spinner = create_spinner("Triggering schedule...");
    let schedule = client.trigger_schedule(id).await.map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!(
        "{} Schedule {} triggered",
        "⚡".cyan(),
        schedule.name.bold()
    );

    Ok(())
}

async fn get_history(
    client: &AllscreenshotsClient,
    id: &str,
    limit: Option<i32>,
) -> CliResult<()> {
    let spinner = create_spinner("Fetching history...");
    let history = client
        .get_schedule_history(id, limit)
        .await
        .map_err(CliError::Sdk)?;
    spinner.finish_and_clear();

    println!(
        "{} ({})",
        "Execution History".bold().underline(),
        format!("{} total", history.total_executions).dimmed()
    );
    println!();

    if history.executions.is_empty() {
        println!("{}", "No executions yet.".dimmed());
        return Ok(());
    }

    for exec in history.executions {
        let status_icon = match exec.status.as_str() {
            "COMPLETED" => "✓".green(),
            "FAILED" => "✗".red(),
            _ => "•".dimmed(),
        };

        println!(
            "{} {} - {}",
            status_icon,
            exec.executed_at,
            exec.status.bold()
        );

        if let Some(ref url) = exec.result_url {
            println!("    Result: {}", url.dimmed());
        }
        if let Some(ref error) = exec.error_message {
            println!("    Error: {}", error.red());
        }
        if let Some(ms) = exec.render_time_ms {
            println!(
                "    Render time: {}",
                crate::utils::format_duration_ms(ms as u64).dimmed()
            );
        }
    }

    Ok(())
}

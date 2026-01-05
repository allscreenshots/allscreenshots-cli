use allscreenshots_sdk::models::{QuotaStatusResponse, UsageResponse};
use colored::Colorize;

/// ASCII graph rendering for usage statistics
pub struct UsageGraph;

impl UsageGraph {
    /// Render a horizontal bar for quota usage
    pub fn render_quota_bar(used: i32, limit: i32, label: &str, bar_width: usize) {
        let percent = if limit > 0 {
            (used as f32 / limit as f32 * 100.0) as i32
        } else {
            0
        };

        let filled = if limit > 0 {
            (bar_width as f32 * used as f32 / limit as f32) as usize
        } else {
            0
        };
        let empty = bar_width.saturating_sub(filled);

        // Color based on usage percentage
        let bar_color = if percent >= 90 {
            "red"
        } else if percent >= 75 {
            "yellow"
        } else {
            "green"
        };

        let filled_bar = "█".repeat(filled);
        let empty_bar = "░".repeat(empty);

        println!("\n{}", label.bold());
        println!(
            "[{}{}] {}/{} ({}%)",
            filled_bar.color(bar_color),
            empty_bar.dimmed(),
            Self::format_number_i32(used),
            Self::format_number_i32(limit),
            percent
        );
    }

    /// Render bandwidth usage bar
    pub fn render_bandwidth_bar(
        used_bytes: i64,
        limit_bytes: i64,
        used_formatted: &str,
        limit_formatted: &str,
        bar_width: usize,
    ) {
        let percent = if limit_bytes > 0 {
            (used_bytes as f64 / limit_bytes as f64 * 100.0) as i32
        } else {
            0
        };

        let filled = if limit_bytes > 0 {
            (bar_width as f64 * used_bytes as f64 / limit_bytes as f64) as usize
        } else {
            0
        };
        let empty = bar_width.saturating_sub(filled);

        let bar_color = if percent >= 90 {
            "red"
        } else if percent >= 75 {
            "yellow"
        } else {
            "green"
        };

        let filled_bar = "█".repeat(filled);
        let empty_bar = "░".repeat(empty);

        println!("\n{}", "Bandwidth".bold());
        println!(
            "[{}{}] {} / {} ({}%)",
            filled_bar.color(bar_color),
            empty_bar.dimmed(),
            used_formatted.green(),
            limit_formatted,
            percent
        );
    }

    /// Render a sparkline from historical data
    pub fn render_sparkline(data: &[i32], label: &str) {
        if data.is_empty() {
            return;
        }

        let chars = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let max = *data.iter().max().unwrap_or(&1) as f32;

        let sparkline: String = data
            .iter()
            .map(|&v| {
                let idx = if max > 0.0 {
                    ((v as f32 / max) * 8.0) as usize
                } else {
                    0
                };
                chars[idx.min(8)]
            })
            .collect();

        println!("\n{}", label.bold());
        println!("{}", sparkline.cyan());
    }

    /// Render complete usage summary
    pub fn render_usage_summary(usage: &UsageResponse) {
        println!("\n{}", "═".repeat(50).dimmed());
        println!("{}", "  API Usage Summary".bold().underline());
        println!("{}", "═".repeat(50).dimmed());

        // Tier
        println!("\n{}: {}", "Tier".bold(), usage.tier.cyan());

        // Current period (not optional)
        let period = &usage.current_period;
        println!(
            "\n{}: {} to {}",
            "Period".bold(),
            period.period_start.dimmed(),
            period.period_end.dimmed()
        );

        // Show current period stats
        println!("\n{}", "Current Period".bold());
        println!(
            "  Screenshots: {}",
            Self::format_number_i32(period.screenshots_count).cyan()
        );
        println!("  Bandwidth: {}", period.bandwidth_formatted.cyan());

        // Quota bars
        if let Some(ref quota) = usage.quota {
            Self::render_quota_bar(
                quota.screenshots.used,
                quota.screenshots.limit,
                "Screenshots",
                40,
            );
            println!(
                "  {} remaining",
                quota.screenshots.remaining.to_string().green()
            );

            Self::render_bandwidth_bar(
                quota.bandwidth.used_bytes,
                quota.bandwidth.limit_bytes,
                &quota.bandwidth.used_formatted,
                &quota.bandwidth.limit_formatted,
                40,
            );
        }

        // History sparkline
        if let Some(ref history) = usage.history {
            if !history.is_empty() {
                let counts: Vec<i32> = history.iter().map(|h| h.screenshots_count).collect();
                Self::render_sparkline(&counts, "Usage History (last periods)");
            }
        }

        // Totals
        if let Some(ref totals) = usage.totals {
            println!("\n{}", "All-Time Totals".bold());
            println!(
                "  Screenshots: {}",
                Self::format_number_i64(totals.screenshots_count).cyan()
            );
            println!("  Bandwidth: {}", totals.bandwidth_formatted.cyan());
        }

        println!("\n{}", "═".repeat(50).dimmed());
    }

    /// Render quota status (simpler view)
    pub fn render_quota_status(quota: &QuotaStatusResponse) {
        println!("\n{}", "Quota Status".bold().underline());
        println!("Tier: {}", quota.tier.cyan());

        Self::render_quota_bar(
            quota.screenshots.used,
            quota.screenshots.limit,
            "Screenshots",
            40,
        );

        // Show remaining
        println!(
            "  {} remaining",
            quota.screenshots.remaining.to_string().green()
        );

        Self::render_bandwidth_bar(
            quota.bandwidth.used_bytes,
            quota.bandwidth.limit_bytes,
            &quota.bandwidth.used_formatted,
            &quota.bandwidth.limit_formatted,
            40,
        );

        if let Some(ref ends) = quota.period_ends {
            println!("\nPeriod ends: {}", ends.dimmed());
        }
    }

    /// Format large numbers with commas (i32)
    fn format_number_i32(n: i32) -> String {
        let s = n.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();

        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 && c != &'-' {
                result.push(',');
            }
            result.push(*c);
        }

        result
    }

    /// Format large numbers with commas (i64)
    fn format_number_i64(n: i64) -> String {
        let s = n.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();

        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 && c != &'-' {
                result.push(',');
            }
            result.push(*c);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(UsageGraph::format_number_i32(1000), "1,000");
        assert_eq!(UsageGraph::format_number_i32(1000000), "1,000,000");
        assert_eq!(UsageGraph::format_number_i32(42), "42");
    }
}

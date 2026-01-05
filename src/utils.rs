use crate::error::{CliError, CliResult};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

/// Validate and normalize a URL
/// Automatically adds https:// if missing
pub fn normalize_url(input: &str) -> CliResult<String> {
    let url_str = if !input.starts_with("http://") && !input.starts_with("https://") {
        format!("https://{}", input)
    } else {
        input.to_string()
    };

    // Validate the URL
    Url::parse(&url_str).map_err(|_| CliError::InvalidUrl(input.to_string()))?;

    Ok(url_str)
}

/// Extract domain from URL for filename generation
pub fn extract_domain(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_else(|| "screenshot".to_string())
        .replace('.', "_")
}

/// Generate an automatic filename based on URL and timestamp
pub fn auto_filename(url: &str, format: &str) -> String {
    let domain = extract_domain(url);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    format!("{}_{}.{}", domain, timestamp, format)
}

/// Read URLs from a file (one per line)
pub fn read_urls_from_file(path: &Path) -> CliResult<Vec<String>> {
    if !path.exists() {
        return Err(CliError::FileNotFound(path.display().to_string()));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| CliError::FileReadError(format!("{}: {}", path.display(), e)))?;

    let urls: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect();

    if urls.is_empty() {
        return Err(CliError::Other(format!(
            "No URLs found in {}",
            path.display()
        )));
    }

    Ok(urls)
}

/// Ensure output directory exists
pub fn ensure_dir(path: &Path) -> CliResult<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| CliError::FileWriteError(format!("Failed to create directory: {}", e)))?;
    }
    Ok(())
}

/// Save bytes to file
pub fn save_to_file(path: &Path, data: &[u8]) -> CliResult<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }

    fs::write(path, data)
        .map_err(|e| CliError::FileWriteError(format!("{}: {}", path.display(), e)))?;

    Ok(())
}

/// Generate output path for batch operations
pub fn batch_output_path(output_dir: &Path, url: &str, index: usize, format: &str) -> PathBuf {
    let domain = extract_domain(url);
    let filename = format!("{:03}_{}.{}", index + 1, domain, format);
    output_dir.join(filename)
}

/// Format file size in human-readable form
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format duration in human-readable form
pub fn format_duration_ms(ms: u64) -> String {
    if ms >= 60000 {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    } else if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

/// List of available device presets
pub fn device_presets() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Desktop HD", "1920x1080"),
        ("Desktop", "1440x900"),
        ("Laptop", "1366x768"),
        ("Tablet Landscape", "1024x768"),
        ("Tablet Portrait", "768x1024"),
        ("iPhone 14 Pro Max", "430x932"),
        ("iPhone 14 Pro", "393x852"),
        ("iPhone 14", "390x844"),
        ("iPhone SE", "375x667"),
        ("iPad Pro 12.9", "1024x1366"),
        ("iPad Pro 11", "834x1194"),
        ("iPad", "820x1180"),
        ("iPad Mini", "744x1133"),
        ("Android Large", "412x915"),
        ("Android Medium", "393x873"),
        ("Android Small", "360x800"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("google.com").unwrap(),
            "https://google.com"
        );
        assert_eq!(
            normalize_url("https://github.com").unwrap(),
            "https://github.com"
        );
        assert_eq!(
            normalize_url("http://example.com").unwrap(),
            "http://example.com"
        );
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.google.com/search"), "www_google_com");
        assert_eq!(extract_domain("https://github.com"), "github_com");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 bytes");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(1500), "1.5s");
        assert_eq!(format_duration_ms(65000), "1m 5s");
    }
}

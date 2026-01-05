use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// Spinner style presets
pub enum SpinnerStyle {
    Dots,
    Braille,
    Line,
    Arrow,
}

impl SpinnerStyle {
    fn tick_chars(&self) -> &'static str {
        match self {
            SpinnerStyle::Dots => "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏",
            SpinnerStyle::Braille => "⣾⣽⣻⢿⡿⣟⣯⣷",
            SpinnerStyle::Line => "|/-\\",
            SpinnerStyle::Arrow => "←↖↑↗→↘↓↙",
        }
    }
}

/// Create a spinner for single operations
pub fn create_spinner(message: &str) -> ProgressBar {
    create_spinner_with_style(message, SpinnerStyle::Dots)
}

/// Create a spinner with a specific style
pub fn create_spinner_with_style(message: &str, style: SpinnerStyle) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars(style.tick_chars())
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a progress bar for operations with known length
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{bar:40.cyan/blue} {pos}/{len} [{elapsed_precise}] ETA: {eta}")
            .unwrap()
            .progress_chars("━━╺"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Create a progress bar with percentage display
pub fn create_percent_bar(message: &str) -> ProgressBar {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{bar:40.cyan/blue} {percent}% [{elapsed_precise}]")
            .unwrap()
            .progress_chars("━━╺"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Create a multi-progress for concurrent operations
pub fn create_multi_progress() -> MultiProgress {
    MultiProgress::new()
}

/// Spinner messages for different operations
pub mod messages {
    pub const CAPTURING: &str = "Capturing screenshot...";
    pub const PROCESSING: &str = "Processing...";
    pub const DOWNLOADING: &str = "Downloading result...";
    pub const UPLOADING: &str = "Uploading...";
    pub const WAITING: &str = "Waiting for job to complete...";
    pub const COMPOSING: &str = "Composing screenshots...";
    pub const SAVING: &str = "Saving to disk...";
}

/// Progress bar helper for batch operations
pub struct BatchProgress {
    multi: MultiProgress,
    main_bar: ProgressBar,
}

impl BatchProgress {
    pub fn new(total: u64, message: &str) -> Self {
        let multi = MultiProgress::new();
        let main_bar = multi.add(create_progress_bar(total, message));

        Self { multi, main_bar }
    }

    pub fn add_spinner(&self, message: &str) -> ProgressBar {
        let spinner = create_spinner(message);
        self.multi.add(spinner)
    }

    pub fn inc(&self, delta: u64) {
        self.main_bar.inc(delta);
    }

    pub fn set_message(&self, message: &str) {
        self.main_bar.set_message(message.to_string());
    }

    pub fn finish_with_message(&self, message: &str) {
        self.main_bar.finish_with_message(message.to_string());
    }

    pub fn finish(&self) {
        self.main_bar.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_spinner() {
        let spinner = create_spinner("Test message");
        assert_eq!(spinner.message(), "Test message");
    }

    #[test]
    fn test_create_progress_bar() {
        let bar = create_progress_bar(100, "Test");
        assert_eq!(bar.length(), Some(100));
    }
}

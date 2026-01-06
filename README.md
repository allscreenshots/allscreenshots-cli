# AllScreenshots CLI

A command-line tool for capturing website screenshots using the [AllScreenshots API](https://dashboard.allscreenshots.com). Built in Rust for performance and reliability.

## Features

- Quick screenshot capture with terminal preview
- Batch processing of multiple URLs
- Device presets (desktop, tablet, mobile)
- Full-page capture and dark mode
- Ad and cookie blocking
- API usage tracking
- Async job management
- Shell completion support

## Installation

### From source

```bash
cargo install --path .
```

### Requirements

- Rust 1.70 or later
- An AllScreenshots API key ([get one here](https://dashboard.allscreenshots.com))

## Quick start

```bash
# Set your API key
export ALLSCREENSHOTS_API_KEY="your-api-key"

# Take a screenshot
allscreenshots https://example.com

# Save to file
allscreenshots https://github.com -o github.png

# Mobile device capture
allscreenshots https://example.com --device "iPhone 14" --full-page
```

## Commands

| Command | Description |
|---------|-------------|
| `capture` | Take a screenshot with custom options |
| `async` | Take async screenshots with job tracking |
| `batch` | Capture multiple URLs in bulk |
| `compose` | Combine multiple screenshots into one image |
| `schedule` | Manage scheduled screenshot tasks |
| `usage` | Display API usage and quota |
| `config` | Manage authentication and settings |
| `jobs` | List and manage screenshot jobs |
| `gallery` | Browse screenshots with previews |
| `watch` | Re-capture at specified intervals |
| `devices` | Show available device presets |
| `completions` | Generate shell completions |

## Configuration

### API key

Set your API key using one of these methods (in order of priority):

1. CLI argument: `-k` or `--api-key`
2. Environment variable: `ALLSCREENSHOTS_API_KEY`
3. Config file: `allscreenshots config add-authtoken <key>`

### Config file

Located at `~/.config/allscreenshots/cli/config.toml` (Linux/macOS):

```toml
[auth]
api_key = "your-api-key"

[defaults]
device = "Desktop HD"
format = "png"
output_dir = "./screenshots"
display = true

[display]
protocol = "auto"
width = 80
height = 24
```

## Capture options

```
--device <DEVICE>     Device preset (e.g., "iPhone 14", "Desktop HD")
--width <WIDTH>       Viewport width in pixels
--height <HEIGHT>     Viewport height in pixels
--format <FORMAT>     Output format: png, jpeg, webp, pdf
--quality <QUALITY>   Image quality (1-100, for jpeg/webp)
--full-page           Capture the entire page
--dark-mode           Enable dark mode
--delay <MS>          Wait before capture
--wait-until <EVENT>  Wait for: load, domcontentloaded, networkidle
--selector <CSS>      Capture specific element
--block-ads           Block advertisements
--block-cookies       Block cookie banners
--custom-css <CSS>    Inject custom CSS
```

## Examples

### Batch capture from file

```bash
# urls.txt contains one URL per line
allscreenshots batch -f urls.txt -o ./screenshots/
```

### Check API usage

```bash
allscreenshots usage
```

### Generate shell completions

```bash
# For bash
allscreenshots completions bash > ~/.bash_completion.d/allscreenshots

# For zsh
allscreenshots completions zsh > ~/.zfunc/_allscreenshots

# For fish
allscreenshots completions fish > ~/.config/fish/completions/allscreenshots.fish
```

## Global options

```
-k, --api-key <KEY>   Override API key
-o, --output <PATH>   Output file path
--display             Show image in terminal
--no-display          Don't show image in terminal
-v, --verbose         Enable verbose output
--json                Output in JSON format
--no-color            Disable colored output
```

## License

Apache-2.0

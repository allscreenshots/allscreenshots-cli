use clap::{Command, CommandFactory};
use clap_complete::{generate, Shell};
use std::io;
use crate::error::{CliError, CliResult};

/// Generate shell completions
pub fn generate_completions(shell: Shell, cmd: &mut Command) -> CliResult<()> {
    let name = cmd.get_name().to_string();
    generate(shell, cmd, name, &mut io::stdout());
    Ok(())
}

/// Print installation instructions for shell completions
pub fn print_instructions(shell: Shell) {
    match shell {
        Shell::Bash => {
            println!("# Add to ~/.bashrc or ~/.bash_profile:");
            println!("eval \"$(allscreenshots completions bash)\"");
            println!();
            println!("# Or save to a file:");
            println!("allscreenshots completions bash > /etc/bash_completion.d/allscreenshots");
        }
        Shell::Zsh => {
            println!("# Add to ~/.zshrc:");
            println!("eval \"$(allscreenshots completions zsh)\"");
            println!();
            println!("# Or save to completions directory:");
            println!("allscreenshots completions zsh > ~/.zsh/completions/_allscreenshots");
            println!();
            println!("# Make sure completions are enabled in ~/.zshrc:");
            println!("autoload -Uz compinit && compinit");
        }
        Shell::Fish => {
            println!("# Save to fish completions directory:");
            println!("allscreenshots completions fish > ~/.config/fish/completions/allscreenshots.fish");
        }
        Shell::PowerShell => {
            println!("# Add to your PowerShell profile:");
            println!("allscreenshots completions powershell | Out-String | Invoke-Expression");
            println!();
            println!("# Or add to $PROFILE:");
            println!("allscreenshots completions powershell >> $PROFILE");
        }
        Shell::Elvish => {
            println!("# Add to ~/.elvish/rc.elv:");
            println!("eval (allscreenshots completions elvish | slurp)");
        }
        _ => {
            println!("# Run the following command and follow your shell's instructions:");
            println!("allscreenshots completions {:?}", shell);
        }
    }
}

/// Parse shell name from string
pub fn parse_shell(s: &str) -> CliResult<Shell> {
    match s.to_lowercase().as_str() {
        "bash" => Ok(Shell::Bash),
        "zsh" => Ok(Shell::Zsh),
        "fish" => Ok(Shell::Fish),
        "powershell" | "ps" => Ok(Shell::PowerShell),
        "elvish" => Ok(Shell::Elvish),
        _ => Err(CliError::Other(format!(
            "Unknown shell '{}'. Supported: bash, zsh, fish, powershell, elvish",
            s
        ))),
    }
}

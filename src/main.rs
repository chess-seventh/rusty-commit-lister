use clap::{Arg, Command};
use anyhow::{Context, Result};
use tracing::{info, warn, error};
use tracing_subscriber;
use rusty_commit_lister::config::Settings;
fn main() -> Result<()> {
// Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting rusty-commit-lister");
// Load configuration
    let settings = Settings::new()
        .context("Failed to load configuration")?;
let matches = Command::new("rusty-commit-lister")
        .version(env!("CARGO_PKG_VERSION"))
        .author(" <>")
        .about("")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count)
                .help("Increase verbosity")
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
        )
        .get_matches();

    let verbosity = matches.get_count("verbose");

    if verbosity > 0 {
        info!("Verbose mode enabled (level: {})", verbosity);
    }
// Main application logic
    run(settings)?;

info!("rusty-commit-lister completed successfully");
Ok(())
}

fn run(_settings: Settings) -> Result<()> {
println!("🦀 Welcome to rusty-commit-lister!");
    println!("This is a Rust binary created with DevBootstrapper");

    // TODO: Implement your application logic here

Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let settings = Settings::default();
        assert!(run(settings).is_ok());
        }
}

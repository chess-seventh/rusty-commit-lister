use anyhow::Result;
use clap::{Arg, Command};
use rusty_commit_lister::RustyCommitLister;
use tracing::info;
fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting rusty-commit-lister");
    // Load configuration
    // let settings = Settings::new().context("Failed to load configuration")?;
    let matches = Command::new("rusty-commit-lister")
        .version(env!("CARGO_PKG_VERSION"))
        .author(" <>")
        .about("")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count)
                .help("Increase verbosity"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file"),
        )
        .get_matches();

    let verbosity = matches.get_count("verbose");

    if verbosity > 0 {
        info!("Verbose mode enabled (level: {})", verbosity);
    }

    let rcl = RustyCommitLister::new();

    // Main application logic
    run(rcl)?;

    info!("rusty-commit-lister completed successfully");
    Ok(())
}

fn run(rusty_commit_lister: RustyCommitLister) -> Result<()> {
    println!("🦀 Welcome to rusty-commit-lister!");
    println!("This is a Rust binary created with DevBootstrapper");

    // TODO: Implement your application logic here

    match rusty_commit_lister.hello() {
        Some(_msg) => Ok(()),
        _ => panic!("not a string"),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_run() {
//         let _run_ret = run();
//         assert!(run(settings).is_ok());
//     }
// }

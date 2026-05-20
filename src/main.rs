// Composition root — wire adapters → probe → run TUI.
//
// CLI flags: --config <path>, --verbose
// Exit codes: 0 = success, 1 = vault error, 2 = config/usage error, 130 = SIGINT

use std::io::IsTerminal;

use anyhow::Result;
use clap::{Arg, Command};
use tracing::info;

use rusty_commit_lister::adapters::arboard_clipboard::ArboardClipboardAdapter;
use rusty_commit_lister::ports::clipboard_port::ClipboardPort;
use rusty_commit_lister::ports::config_port::ConfigPort;
use rusty_commit_lister::ports::config_port::Probe;
use rusty_commit_lister::ports::vault_port::VaultScanPort;

/// Resolves the default config file path: `~/.config/rusty-commit-lister/config.toml`.
fn default_config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
    std::path::PathBuf::from(home)
        .join(".config")
        .join("rusty-commit-lister")
        .join("config.toml")
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting rusty-commit-lister");

    let matches = Command::new("rusty-commit-lister")
        .version(env!("CARGO_PKG_VERSION"))
        .author("franci")
        .about("Browse commits from Obsidian daily notes in a terminal TUI")
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
                .help("Path to config.toml (overrides ~/.config/rusty-commit-lister/config.toml)"),
        )
        .get_matches();

    let verbosity = matches.get_count("verbose");
    if verbosity > 0 {
        info!("Verbose mode enabled (level: {})", verbosity);
    }

    // 1. Determine config_path from --config flag or default
    let config_path = matches
        .get_one::<String>("config")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(default_config_path);

    // 2. Load config — detect absence before loading so we can give a graceful notice
    let config_absent = !config_path.exists();
    let config_path_display = config_path.display().to_string();
    let config_adapter =
        rusty_commit_lister::adapters::toml_config::TomlConfigAdapter::new(config_path);
    let config = config_adapter.load().unwrap_or_else(|e| {
        eprintln!("Configuration error: {e}");
        std::process::exit(2);
    });

    if config_absent {
        println!("No config file found at {config_path_display}, using defaults");
        return Ok(());
    }

    // 3. Probe vault (fatal)
    let vault = rusty_commit_lister::adapters::walkdir_vault::WalkdirScanAdapter::new(
        config.vault_path.clone(),
    );
    vault.probe().unwrap_or_else(|e| {
        eprintln!("Vault error: {e}");
        std::process::exit(1);
    });

    // 4. Probe clipboard (non-fatal — degrades gracefully)
    let clipboard_available = ArboardClipboardAdapter::new().probe().is_ok();
    let mut config = config;
    config.clipboard_available = clipboard_available;
    if !clipboard_available {
        tracing::warn!("Clipboard unavailable — copy feature degraded");
    }

    // 5. Scan vault
    let records = vault.scan(config.scan_days_back).unwrap_or_else(|e| {
        tracing::warn!(%e, "vault scan failed");
        vec![]
    });

    // 6. Build model
    let scan_days_back = config.scan_days_back;
    let initial_model = rusty_commit_lister::domain::model::AppModel::new(config);
    let model = rusty_commit_lister::domain::update::update(
        initial_model,
        rusty_commit_lister::domain::events::AppEvent::LoadComplete(records),
    );

    // 7. TTY detection and run
    if std::io::stdout().is_terminal() {
        let mut tui = rusty_commit_lister::tui::event_loop::TuiEventLoop::new()?;
        tui.run(
            model,
            move || {
                vault
                    .scan(scan_days_back)
                    .unwrap_or_else(|e| {
                        tracing::warn!(%e, "reload failed");
                        vec![]
                    })
            },
            |url| {
                ArboardClipboardAdapter::new()
                    .write(url)
                    .map_err(|e| e.to_string())
            },
        )?;
    } else {
        if model.commit_rows.is_empty() {
            println!(
                "No commits found in the last {} days",
                model.config.scan_days_back
            );
        } else {
            println!("Found {} commits:", model.commit_rows.len());
            for r in &model.commit_rows {
                println!("  {} {} - {}", r.date, r.time, r.message);
            }
        }
    }

    Ok(())
}

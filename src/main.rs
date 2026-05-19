// SCAFFOLD: true
// Composition root — wire adapters → probe → run TUI.
// Real implementation replaces this body in DELIVER wave.
//
// CLI flags: --config <path>, --verbose, --date <YYYY-MM-DD>, --today, --daily-note <path>
// Exit codes: 0 = success, 1 = runtime error, 2 = config/usage error, 130 = SIGINT

use anyhow::Result;
use clap::{Arg, Command};
use tracing::info;

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

    // SCAFFOLD: true
    // Real composition root:
    //   1. Determine config_path from --config flag or default
    //   2. TomlConfigAdapter::new(config_path).load() → AppConfig
    //   3. WalkdirScanAdapter::new(config.vault_path).probe() → verify vault
    //   4. ArboardClipboardAdapter::new().probe() → set clipboard_available (non-fatal)
    //   5. VaultScanPort::scan(config.scan_days_back) → Vec<CommitRecord>
    //   6. AppModel::new(config) + update(model, LoadComplete(records))
    //   7. TuiEventLoop::new()?.run(model)

    panic!("Not yet implemented -- RED scaffold")
}

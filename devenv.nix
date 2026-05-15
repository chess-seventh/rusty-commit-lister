{ pkgs, config, inputs, lib, ... }: {

  # Environment variables
  dotenv = {
    enable = true;
    filename = [ ".env" ];
  };

  difftastic.enable = true;

  # Rust language support
  languages.rust = {
    enable = true;
    channel = "nightly";

    # Rust components
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "rust-src"
      "rust-std"
      "llvm-tools-preview"
    ];

    # Targets for cross-compilation
    targets = [ "x86_64-unknown-linux-gnu" ];
  };

  # Development packages
  packages = with pkgs; [
    # Rust development tools
    cargo-edit # cargo add, cargo rm, cargo upgrade
    cargo-watch # cargo watch for auto-rebuild
    cargo-expand # cargo expand for macro debugging
    cargo-outdated # check for outdated dependencies
    cargo-audit # security audit
    cargo-deny # dependency management
    cargo-release # release management
    cargo-cross # cross-compilation
    cargo-nextest # next-generation test runner
    cargo-llvm-cov # code coverage
    cargo-machete # find unused dependencies
    cargo-update # update installed binaries
    bacon

    # Build tools and linkers
    gcc
    binutils
    pkg-config
    openssl

    # Development utilities
    git
    jq
    tree
    curl
    gh
    glab

    # Container tools
    docker
    docker-compose
  ];

  # Git hooks
  git-hooks.hooks = {
    rusty-commit-saver = {
      enable = true;
      name = "🦀 Rusty Commit Saver";
      stages = [ "post-commit" ];
      after = [ "commitizen" "gitlint" "gptcommit" ];
      entry = "${
          inputs.rusty-commit-saver.packages.${pkgs.system}.default
        }/bin/rusty-commit-saver";
      pass_filenames = false;
      language = "system";
      always_run = true;
    };

    check-merge-conflicts = {
      name = "🔒 Check Merge Conflicts";
      enable = true;
      stages = [ "pre-commit" ];
    };

    detect-aws-credentials = {
      name = "💭 Detect AWS Credentials";
      enable = true;
      stages = [ "pre-commit" ];
    };

    detect-private-keys = {
      name = "🔑 Detect Private Keys";
      enable = true;
      stages = [ "pre-commit" ];
    };

    end-of-file-fixer = {
      name = "🔚 End of File Fixer";
      enable = true;
      stages = [ "pre-commit" ];
    };

    mixed-line-endings = {
      name = "🔀 Mixed Line Endings";
      enable = true;
      stages = [ "pre-commit" ];
    };

    trim-trailing-whitespace = {
      name = "✨ Trim Trailing Whitespace";
      enable = true;
      stages = [ "pre-commit" ];
    };

    shellcheck = {
      name = "✨ Shell Check";
      enable = true;
      stages = [ "pre-commit" ];
    };

    mdsh = {
      enable = true;
      name = "✨ MDSH";
      stages = [ "pre-commit" ];
    };

    treefmt = {
      name = "🌲 TreeFMT";
      enable = true;
      settings.formatters = [
        pkgs.nixfmt-classic
        pkgs.deadnix
        pkgs.yamlfmt
        pkgs.rustfmt
        pkgs.toml-sort
      ];
      stages = [ "pre-commit" ];
    };

    commitizen = {
      name = "✨ Commitizen";
      enable = true;
      stages = [ "post-commit" ];
    };

    gptcommit = {
      name = "🤖 GPT Commit";
      enable = true;
    };

    gitlint = {
      name = "✨ GitLint";
      enable = true;
      after = [ "gptcommit" ];
    };

    markdownlint = {
      name = "✨ MarkdownLint";
      enable = true;
      stages = [ "pre-commit" ];
      settings.configuration = {
        MD033 = false;
        MD013 = {
          line_length = 120;
          tables = false;
        };
        MD041 = false;
      };
    };

    # Rust-specific hooks
    rustfmt = {
      name = "🦀 Rust Format";
      enable = true;
      stages = [ "pre-commit" ];
    };

    clippy = {
      name = "🦀 Clippy";
      enable = true;
      stages = [ "pre-commit" ];
      args = [ "--" "-W" "clippy::pedantic" ];
    };
  };

  # Development scripts
  scripts = {
    hello = {
      description = "Welcome message for Rust project";
      exec = ''
        figlet "rusty-commit-lister" -w 1000
        echo "🦀 Welcome to your Rust development environment!"
        echo "   Project: rusty-commit-lister"
        echo "   Crate: rusty_commit_lister"
        echo ""
        echo "🔧 Rust toolchain information:"
        rustc --version
        cargo --version
        echo ""
      '';
    };

    install_pre_hooks = {
      description = "Install and configure pre-commit hooks";
      exec = ''
        #!/usr/bin/env bash
        set -euxo pipefail
        gptcommit install
        gptcommit config set openai.model gpt-4-turbo
        gptcommit config set output.conventional_commit true
      '';
    };

    prepare_git_repo = {
      description = "One time script to bootstrap the github/gitlab repository";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail

        if git rev-parse --git-dir > /dev/null 2>&1 && git remote get-url origin > /dev/null 2>&1; then
            echo "Repository with remote origin exists"
        else
            echo "Not a repository or no remote configured"

            if [[ "$PWD" == *"github.com"* ]]; then
              echo "Current directory contains github.com"
              gh repo new "rusty-commit-lister" --public
            elif [[ "$PWD" == *"gitlab.com"* ]]; then
              echo "Current directory contains gitlab.com"
              glab repo create --name "rusty-commit-lister"
            else
              echo "Current directory is niether github or gitlab, manual operation is needed"
            fi
        fi

      '';
    };

    build = {
      description = "Build the Rust project";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🔨 Building Rust project..."
        cargo build
      '';
    };

    build-release = {
      description = "Build the Rust project in release mode";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🚀 Building Rust project (release mode)..."
        cargo build --release
      '';
    };

    test = {
      description = "Run tests with cargo nextest";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🧪 Running tests..."
        cargo nextest run --no-fail-fast --all-targets
      '';
    };

    test-coverage = {
      description = "Run tests with coverage";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📊 Running tests with coverage..."
        cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
        cargo llvm-cov report
      '';
    };

    format = {
      description = "Format code with rustfmt";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🎨 Formatting code..."
        cargo fmt
      '';
    };

    check = {
      description = "Check code without building";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "✅ Checking code..."
        cargo check
      '';
    };

    audit = {
      description = "Security audit with cargo audit";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🔒 Running security audit..."
        cargo audit
      '';
    };

    outdated = {
      description = "Check for outdated dependencies";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📦 Checking for outdated dependencies..."
        cargo outdated
      '';
    };

    watch = {
      description = "Watch for changes and rebuild";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "👀 Watching for changes..."
        cargo watch -x build
      '';
    };

    watch-clippy = {
      description = "Watch and re-run tests on file changes";
      exec = ''
        bacon clippy
      '';
    };

    watch-coverage = {
      description = "Watch and re-run nextest on file changes";
      exec = ''
        bacon coverage
      '';
    };

    run = {
      description = "Run the binary";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🚀 Running rusty-commit-lister..."
        cargo run
      '';
    };

    clean = {
      description = "Clean build artifacts";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "🧹 Cleaning build artifacts..."
        cargo clean
      '';
    };

    deps = {
      description = "Show dependency tree";
      exec = ''
        #!/usr/bin/env bash
        set -euo pipefail
        echo "📦 Dependency tree:"
        cargo tree
      '';
    };

    devhelp = {
      description = "Returns the helper comamnds";
      exec = ''
        echo
        echo 💡 Helper scripts for DevBootstrapper development:
        echo
        ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
        ${lib.generators.toKeyValue { }
        (lib.mapAttrs (_name: value: value.description) config.scripts)}
        EOF
        echo
      '';
    };
  };

  # Enter shell configuration
  enterShell = ''
    if [ -f ".env" ]; then
      export $(cat .env | xargs)
    fi

    # Set environment variables for Rust development
    export GREET="rusty-commit-lister"
    export RUST_LOG="info"
    export RUST_BACKTRACE="1"

    # Convenient aliases
    alias c=check
    alias b=build
    alias br=build-release
    alias t=test
    alias tc=test-coverage
    alias l=lint
    alias f=format
    alias r=run
    alias w=watch-coverage
    alias wcc=watch-clippy
    alias wt=watch-test

    # Welcome message
    hello
    install_pre_hooks

    echo
    echo 💡 Helper scripts for Rust development:
    echo
    ${pkgs.gnused}/bin/sed -e 's| |••|g' -e 's|=| |' <<EOF | ${pkgs.util-linuxMinimal}/bin/column -t | ${pkgs.gnused}/bin/sed -e 's|^|• |' -e 's|••| |g'
    ${lib.generators.toKeyValue { }
    (lib.mapAttrs (_name: value: value.description) config.scripts)}
    EOF
    echo
  '';

  enterTest = ''
    cargo clippy --all-targets -- -D warnings
    cargo llvm-cov --html nextest --no-fail-fast
    cargo nextest run --no-fail-fast --all-targets
  '';
}

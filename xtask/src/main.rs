//! Build automation for curvine-kube
//!
//! Usage: cargo xtask <command>
//!
//! Available commands:
//! - build: Build the project
//! - test: Run tests
//! - dist: Create distribution packages
//! - install: Install to system
//! - ci: Run CI checks

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build automation for curvine-kube")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Run tests
    Test {
        /// Run only integration tests
        #[arg(long)]
        integration: bool,
    },
    /// Create distribution packages
    Dist {
        /// Target triple (e.g., x86_64-unknown-linux-gnu)
        #[arg(long)]
        target: Option<String>,
    },
    /// Install to system
    Install {
        /// Installation prefix (default: /usr/local)
        #[arg(long, default_value = "/usr/local")]
        prefix: String,
    },
    /// Run CI checks (format, clippy, test)
    Ci,
    /// Format code
    Format {
        /// Check formatting without modifying files
        #[arg(long)]
        check: bool,
    },
    /// Run clippy
    Clippy,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    sh.change_dir(project_root());

    match cli.command {
        Commands::Build { release } => build(&sh, release),
        Commands::Test { integration } => test(&sh, integration),
        Commands::Dist { target } => dist(&sh, target),
        Commands::Install { prefix } => install(&sh, &prefix),
        Commands::Ci => ci(&sh),
        Commands::Format { check } => format(&sh, check),
        Commands::Clippy => clippy(&sh),
    }
}

fn build(sh: &Shell, release: bool) -> Result<()> {
    println!("🔨 Building curvine-kube...");

    if release {
        cmd!(sh, "cargo build --release").run()?;
        println!("✅ Release build completed: target/release/curvine-kube");
    } else {
        cmd!(sh, "cargo build").run()?;
        println!("✅ Debug build completed: target/debug/curvine-kube");
    }

    Ok(())
}

fn test(sh: &Shell, integration: bool) -> Result<()> {
    println!("🧪 Running tests...");

    if integration {
        cmd!(sh, "cargo test --test '*'").run()?;
    } else {
        cmd!(sh, "cargo test --all").run()?;
    }

    println!("✅ All tests passed");
    Ok(())
}

fn dist(sh: &Shell, target: Option<String>) -> Result<()> {
    println!("📦 Creating distribution package...");

    // Build release binary
    if let Some(ref target_triple) = target {
        cmd!(sh, "cargo build --release --target {target_triple}").run()?;
    } else {
        cmd!(sh, "cargo build --release").run()?;
    }

    // Create dist directory
    let dist_dir = project_root().join("dist");
    sh.create_dir(&dist_dir)?;

    // Copy binary
    let binary_src = if let Some(ref target_triple) = target {
        project_root().join(format!("target/{}/release/curvine-kube", target_triple))
    } else {
        project_root().join("target/release/curvine-kube")
    };

    let binary_dst = dist_dir.join("curvine-kube");
    sh.copy_file(&binary_src, &binary_dst)?;

    // Create tarball
    let version = env!("CARGO_PKG_VERSION");
    let archive_name = format!("curvine-kube-{}.tar.gz", version);

    cmd!(sh, "tar -czf {archive_name} -C dist curvine-kube")
        .run()
        .context("Failed to create tarball")?;

    println!("✅ Distribution package created: {}", archive_name);
    Ok(())
}

fn install(sh: &Shell, prefix: &str) -> Result<()> {
    println!("📥 Installing curvine-kube to {}...", prefix);

    // Build release if not exists
    let binary = project_root().join("target/release/curvine-kube");
    if !binary.exists() {
        println!("Building release binary first...");
        cmd!(sh, "cargo build --release").run()?;
    }

    // Install binary
    let bin_dir = Path::new(prefix).join("bin");
    sh.create_dir(&bin_dir)?;

    let install_path = bin_dir.join("curvine-kube");
    sh.copy_file(&binary, &install_path)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&install_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("✅ Installed to: {}", install_path.display());
    Ok(())
}

fn ci(sh: &Shell) -> Result<()> {
    println!("🔍 Running CI checks...");

    println!("\n📝 Checking formatting...");
    format(sh, true)?;

    println!("\n🔧 Running clippy...");
    clippy(sh)?;

    println!("\n🧪 Running tests...");
    test(sh, false)?;

    println!("\n✅ All CI checks passed!");
    Ok(())
}

fn format(sh: &Shell, check: bool) -> Result<()> {
    if check {
        cmd!(sh, "cargo fmt --all -- --check").run()?;
        println!("✅ Code formatting is correct");
    } else {
        cmd!(sh, "cargo fmt --all").run()?;
        println!("✅ Code formatted");
    }
    Ok(())
}

fn clippy(sh: &Shell) -> Result<()> {
    cmd!(
        sh,
        "cargo clippy --all-targets --all-features -- -D warnings"
    )
    .run()?;
    println!("✅ Clippy checks passed");
    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

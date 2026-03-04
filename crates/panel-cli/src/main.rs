//! Panel1 - Linux Server Management Panel with TUI

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "panel1")]
#[command(about = "Panel1 - Linux Server Management Panel", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start simplified TUI interface (default)
    Tui,
    /// Show system status
    Status,
    /// Install a tool service from URL (agent mode)
    Install {
        /// Download URL for binary/archive
        url: String,
        /// Optional service name override
        #[arg(short, long)]
        name: Option<String>,
        /// Show detailed install logs
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Tui) => {
            handle_tui_command().await?;
        }
        Some(Commands::Status) => {
            show_system_status()?;
        }
        Some(Commands::Install { url, name, verbose }) => {
            handle_install_command(&url, name.as_deref(), verbose).await?;
        }
    }

    Ok(())
}

fn show_system_status() -> Result<()> {
    let mut monitor = panel_core::SystemMonitor::new();
    let info = monitor.get_system_info();
    let cpu = monitor.get_cpu_info();
    let memory = monitor.get_memory_info();
    let disks = monitor.get_disk_info();

    println!("=== System Status ===");
    println!("Hostname: {}", info.hostname);
    println!("OS: {} {}", info.os_name, info.os_version);
    println!("Kernel: {}", info.kernel_version);
    println!("Uptime: {} seconds", info.uptime);
    println!();
    println!("CPU: {} ({} cores)", cpu.brand, cpu.cores);
    println!("CPU Usage: {:.1}%", cpu.usage);
    println!();
    println!(
        "Memory: {:.1} GB / {:.1} GB",
        memory.used as f64 / 1024.0 / 1024.0 / 1024.0,
        memory.total as f64 / 1024.0 / 1024.0 / 1024.0
    );
    println!("Memory Usage: {:.1}%", memory.usage);
    println!();
    println!("=== Disk Usage ===");
    for disk in disks {
        println!(
            "{} ({}) {:.1} GB / {:.1} GB ({:.1}%)",
            disk.mount_point,
            disk.fs_type,
            disk.used as f64 / 1024.0 / 1024.0 / 1024.0,
            disk.total as f64 / 1024.0 / 1024.0 / 1024.0,
            disk.usage
        );
    }

    Ok(())
}

async fn handle_tui_command() -> Result<()> {
    panel_tui::run_tui().await
}

async fn handle_install_command(
    url: &str,
    preferred_name: Option<&str>,
    verbose: bool,
) -> Result<()> {
    let provider: Arc<dyn panel_ai::LlmProvider> = Arc::new(panel_ai::ClaudeProvider::new());
    let installer = panel_ai::InstallerAgent::new(provider);

    println!("Installing from URL...");

    let report = installer.install_from_url(url, preferred_name).await?;

    if verbose {
        for line in &report.logs {
            println!("- {}", line);
        }
    }

    if report.success {
        println!("Install finished successfully.");
        if let Some(name) = report.service_name {
            println!("Service: {}", name);
        }
        if let Some(path) = report.binary_path {
            println!("Binary: {}", path);
        }
        Ok(())
    } else {
        bail!(
            "Install failed: {}",
            report.error.unwrap_or_else(|| "unknown error".to_string())
        )
    }
}

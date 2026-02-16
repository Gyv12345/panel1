//! Panel1 - Linux Server Management Panel with TUI

use anyhow::Result;
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
    /// Start TUI interface (default)
    Tui {
        /// Start directly in wizard mode
        #[arg(long)]
        wizard: bool,
        /// Start directly in AI chat mode
        #[arg(long)]
        chat: bool,
    },
    /// Show system status
    Status,
    /// Manage system services
    Service {
        #[command(subcommand)]
        action: ServiceCommands,
    },
    /// AI-powered features
    Ai {
        #[command(subcommand)]
        action: AiCommands,
    },
    /// Install and manage services
    Install {
        /// Service name (redis, elasticsearch, nginx, etc.)
        name: String,
        /// Installation mode (systemd, panel1, docker)
        #[arg(short, long, default_value = "panel1")]
        mode: String,
        /// Version to install
        #[arg(short, long)]
        version: Option<String>,
    },
}

#[derive(Subcommand)]
enum ServiceCommands {
    /// List all services
    List,
    /// Start a service
    Start {
        /// Service name
        name: String,
    },
    /// Stop a service
    Stop {
        /// Service name
        name: String,
    },
    /// Restart a service
    Restart {
        /// Service name
        name: String,
    },
}

#[derive(Subcommand)]
enum AiCommands {
    /// Get installation advice for a service
    Install {
        /// Service name
        name: String,
    },
    /// Run system diagnostics
    Diagnose,
    /// Get performance optimization advice
    Optimize,
    /// Run security check
    Security,
    /// Ask a question
    Ask {
        /// The question to ask
        #[arg(trailing_var_arg = true)]
        question: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        None => {
            // 默认启动 TUI
            handle_tui_command(false, false).await?;
        }
        Some(Commands::Status) => {
            show_system_status()?;
        }
        Some(Commands::Service { action }) => {
            handle_service_command(action)?;
        }
        Some(Commands::Tui { wizard, chat }) => {
            handle_tui_command(wizard, chat).await?;
        }
        Some(Commands::Ai { action }) => {
            handle_ai_command(action).await?;
        }
        Some(Commands::Install {
            name,
            mode,
            version,
        }) => {
            handle_install_command(&name, &mode, version.as_deref()).await?;
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

fn handle_service_command(action: ServiceCommands) -> Result<()> {
    let manager = panel_core::ServiceManager::new();
    match action {
        ServiceCommands::List => match manager.get_services() {
            Ok(services) => {
                println!("{:<40} {:<10} {:<10}", "NAME", "STATUS", "ENABLED");
                for service in services {
                    println!(
                        "{:<40} {:<10} {:<10}",
                        service.name,
                        format!("{:?}", service.status),
                        service.enabled
                    );
                }
            }
            Err(e) => {
                eprintln!("Error listing services: {}", e);
            }
        },
        ServiceCommands::Start { name } => match manager.start(&name) {
            Ok(()) => println!("Service {} started", name),
            Err(e) => eprintln!("Error starting service: {}", e),
        },
        ServiceCommands::Stop { name } => match manager.stop(&name) {
            Ok(()) => println!("Service {} stopped", name),
            Err(e) => eprintln!("Error stopping service: {}", e),
        },
        ServiceCommands::Restart { name } => match manager.restart(&name) {
            Ok(()) => println!("Service {} restarted", name),
            Err(e) => eprintln!("Error restarting service: {}", e),
        },
    }
    Ok(())
}

async fn handle_tui_command(wizard: bool, chat: bool) -> Result<()> {
    if wizard {
        println!("Starting Panel1 TUI in wizard mode...");
        panel_tui::run_wizard().await?;
    } else if chat {
        println!("Starting Panel1 TUI in AI chat mode...");
        panel_tui::run_chat().await?;
    } else {
        println!("Starting Panel1 TUI...");
        panel_tui::run_tui().await?;
    }
    Ok(())
}

async fn handle_ai_command(action: AiCommands) -> Result<()> {
    // 初始化 AI Provider
    let provider = create_ai_provider()?;

    match action {
        AiCommands::Install { name } => {
            let agent = panel_ai::InstallerAgent::new(provider);
            println!("Getting installation advice for {}...\n", name);
            let response = agent.get_install_advice(&name).await?;
            println!("{}", response.content);

            if !response.suggested_commands.is_empty() {
                println!("\n=== Suggested Commands ===");
                for cmd in response.suggested_commands {
                    println!("  {}", cmd);
                }
            }
        }
        AiCommands::Diagnose => {
            let agent = panel_ai::AdvisorAgent::new(provider);
            println!("Running system diagnostics...\n");
            let response = agent.diagnose_system().await?;
            println!("{}", response.content);
        }
        AiCommands::Optimize => {
            let agent = panel_ai::AdvisorAgent::new(provider);
            println!("Analyzing system performance...\n");
            let response = agent.get_performance_advice().await?;
            println!("{}", response.content);
        }
        AiCommands::Security => {
            let agent = panel_ai::AdvisorAgent::new(provider);
            println!("Running security check...\n");
            let response = agent.security_check().await?;
            println!("{}", response.content);
        }
        AiCommands::Ask { question } => {
            let question = question.join(" ");
            if question.is_empty() {
                eprintln!("Please provide a question");
                return Ok(());
            }
            let agent = panel_ai::AdvisorAgent::new(provider);
            let response = agent.ask(&question).await?;
            println!("{}", response.content);
        }
    }

    Ok(())
}

async fn handle_install_command(name: &str, mode: &str, version: Option<&str>) -> Result<()> {
    let service_mode = match mode.to_lowercase().as_str() {
        "systemd" => panel_service::ServiceMode::Systemd,
        "panel1" => panel_service::ServiceMode::Panel1,
        "docker" => panel_service::ServiceMode::Docker,
        _ => {
            eprintln!("Unknown mode: {}. Use systemd, panel1, or docker.", mode);
            return Ok(());
        }
    };

    let manager = panel_service::ServiceManager::new();
    let default_version = "latest";

    println!(
        "Installing {} (mode: {}, version: {})...",
        name,
        mode,
        version.unwrap_or(default_version)
    );

    match manager
        .install_service(name, name, service_mode, version.unwrap_or(default_version))
        .await
    {
        Ok(service) => {
            println!("Service installed successfully!");
            println!("  Name: {}", service.name);
            println!("  Type: {}", service.service_type);
            println!("  Mode: {:?}", service.mode);
            println!("  Version: {}", service.version);
            if let Some(ref path) = service.binary_path {
                println!("  Binary: {}", path);
            }
        }
        Err(e) => {
            eprintln!("Failed to install service: {}", e);
        }
    }

    Ok(())
}

fn create_ai_provider() -> Result<Arc<dyn panel_ai::LlmProvider>> {
    // 尝试使用 OpenAI
    if let Ok(provider) = panel_ai::OpenAiProvider::from_env() {
        return Ok(Arc::new(provider));
    }

    // 回退到 Ollama
    println!("Note: OPENAI_API_KEY not set, using Ollama (local model)");
    let provider = panel_ai::OllamaProvider::with_model("llama3");
    Ok(Arc::new(provider))
}

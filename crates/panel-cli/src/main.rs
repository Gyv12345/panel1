//! Panel1 CLI - 命令行工具

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "panel1")]
#[command(about = "Panel1 - Linux Server Management Panel CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server
    Start {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Data directory
        #[arg(short, long, default_value = "/opt/panel/data")]
        data_dir: String,
    },
    /// Create initial admin user
    Setup {
        /// Admin username
        #[arg(short, long)]
        username: String,
        /// Admin password
        #[arg(short, long)]
        password: String,
    },
    /// Show system status
    Status,
    /// Manage services
    Service {
        #[command(subcommand)]
        action: ServiceCommands,
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { port, data_dir } => {
            println!("Starting Panel1 on port {} with data dir: {}", port, data_dir);
            println!("Please use the main binary: panel1");
        }
        Commands::Setup { username, password } => {
            println!("Creating admin user: {}", username);
            // TODO: Implement setup logic
            println!("User created successfully!");
        }
        Commands::Status => {
            let mut monitor = panel_core::SystemMonitor::new();
            let info = monitor.get_system_info();
            let cpu = monitor.get_cpu_info();
            let memory = monitor.get_memory_info();

            println!("=== System Status ===");
            println!("Hostname: {}", info.hostname);
            println!("OS: {} {}", info.os_name, info.os_version);
            println!("Kernel: {}", info.kernel_version);
            println!("Uptime: {} seconds", info.uptime);
            println!();
            println!("CPU: {} ({} cores)", cpu.brand, cpu.cores);
            println!("CPU Usage: {:.1}%", cpu.usage);
            println!();
            println!("Memory: {:.1} GB / {:.1} GB",
                memory.used as f64 / 1024.0 / 1024.0 / 1024.0,
                memory.total as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("Memory Usage: {:.1}%", memory.usage);
        }
        Commands::Service { action } => {
            let manager = panel_core::ServiceManager::new();
            match action {
                ServiceCommands::List => {
                    match manager.get_services() {
                        Ok(services) => {
                            println!("{:<40} {:<10} {:<10}", "NAME", "STATUS", "ENABLED");
                            for service in services {
                                println!("{:<40} {:<10} {:<10}",
                                    service.name,
                                    format!("{:?}", service.status),
                                    service.enabled
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Error listing services: {}", e);
                        }
                    }
                }
                ServiceCommands::Start { name } => {
                    match manager.start(&name) {
                        Ok(()) => println!("Service {} started", name),
                        Err(e) => eprintln!("Error starting service: {}", e),
                    }
                }
                ServiceCommands::Stop { name } => {
                    match manager.stop(&name) {
                        Ok(()) => println!("Service {} stopped", name),
                        Err(e) => eprintln!("Error stopping service: {}", e),
                    }
                }
                ServiceCommands::Restart { name } => {
                    match manager.restart(&name) {
                        Ok(()) => println!("Service {} restarted", name),
                        Err(e) => eprintln!("Error restarting service: {}", e),
                    }
                }
            }
        }
    }

    Ok(())
}

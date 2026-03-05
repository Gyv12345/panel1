//! Panel1 - Linux Server Management Panel with TUI

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
        /// Install strategy: auto detects dependencies, docker enforces Docker path
        #[arg(long, value_enum, default_value_t = InstallModeArg::Auto)]
        mode: InstallModeArg,
        /// Show detailed install logs
        #[arg(short, long)]
        verbose: bool,
    },
    /// Update panel1 binary via official installer script
    Update {
        /// Target version, e.g. v0.1.0 or latest
        #[arg(long)]
        version: Option<String>,
        /// GitHub repo override, e.g. owner/name
        #[arg(long)]
        repo: Option<String>,
        /// Install directory override
        #[arg(long)]
        install_dir: Option<String>,
        /// Disable cargo source fallback
        #[arg(long, default_value_t = false)]
        no_source_fallback: bool,
    },
    /// Configure and inspect AI model settings
    Ai {
        #[command(subcommand)]
        command: AiCommands,
    },
}

#[derive(Subcommand)]
enum AiCommands {
    /// Create or update an AI profile
    Config {
        /// Profile name to create/update (default: current active profile)
        #[arg(long)]
        profile: Option<String>,
        /// Create profile from builtin preset name
        #[arg(long)]
        preset: Option<String>,
        /// Protocol type: openai or anthropic
        #[arg(long, value_enum)]
        protocol: Option<AiProtocolArg>,
        /// Model name (e.g. deepseek-chat / qwen-plus / claude-sonnet-4-5)
        #[arg(long)]
        model: Option<String>,
        /// Custom API base URL
        #[arg(long)]
        base_url: Option<String>,
        /// API key
        #[arg(long)]
        api_key: Option<String>,
        /// Set this profile as active after save
        #[arg(long, default_value_t = true)]
        activate: bool,
    },
    /// Show active profile and all configured profiles
    Show,
    /// Quickly switch model while keeping other fields unchanged
    SetModel {
        /// New model name
        model: String,
        /// Optional profile name (default: active profile)
        #[arg(long)]
        profile: Option<String>,
    },
    /// Manage profiles
    Profiles {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Import builtin provider/model presets
    SeedPresets {
        /// Overwrite existing preset profiles with same name
        #[arg(long, default_value_t = false)]
        overwrite: bool,
    },
}

#[derive(Subcommand)]
enum ProfileCommands {
    /// List profile names and key settings
    List,
    /// Set active profile
    Use { name: String },
    /// Remove one profile
    Remove { name: String },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InstallModeArg {
    Auto,
    Panel1,
    Docker,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AiProtocolArg {
    Openai,
    Anthropic,
}

#[derive(Debug, Default)]
struct AiConfigOverrides {
    profile_name: Option<String>,
    preset_name: Option<String>,
    protocol: Option<panel_ai::AiProtocol>,
    model: Option<String>,
    base_url: Option<String>,
    api_key: Option<String>,
    activate: bool,
}

impl From<InstallModeArg> for panel_ai::InstallMode {
    /// 从输入值转换为目标类型。
    fn from(value: InstallModeArg) -> Self {
        match value {
            InstallModeArg::Auto => panel_ai::InstallMode::Auto,
            InstallModeArg::Panel1 => panel_ai::InstallMode::Panel1,
            InstallModeArg::Docker => panel_ai::InstallMode::Docker,
        }
    }
}

impl From<AiProtocolArg> for panel_ai::AiProtocol {
    /// 从输入值转换为目标类型。
    fn from(value: AiProtocolArg) -> Self {
        match value {
            AiProtocolArg::Openai => panel_ai::AiProtocol::Openai,
            AiProtocolArg::Anthropic => panel_ai::AiProtocol::Anthropic,
        }
    }
}

/// 程序主入口。
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
        Some(Commands::Install {
            url,
            name,
            mode,
            verbose,
        }) => {
            handle_install_command(&url, name.as_deref(), mode.into(), verbose).await?;
        }
        Some(Commands::Update {
            version,
            repo,
            install_dir,
            no_source_fallback,
        }) => {
            handle_update_command(
                version.as_deref(),
                repo.as_deref(),
                install_dir.as_deref(),
                no_source_fallback,
            )?;
        }
        Some(Commands::Ai { command }) => {
            handle_ai_command(command)?;
        }
    }

    Ok(())
}

/// 输出当前系统状态。
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

/// 启动 TUI。
async fn handle_tui_command() -> Result<()> {
    panel_tui::run_tui().await
}

/// 执行 URL 安装流程。
async fn handle_install_command(
    url: &str,
    preferred_name: Option<&str>,
    mode: panel_ai::InstallMode,
    verbose: bool,
) -> Result<()> {
    ensure_ai_config_for_first_time_use()?;

    let provider: Arc<dyn panel_ai::LlmProvider> = Arc::new(panel_ai::ClaudeProvider::new());
    let installer = panel_ai::InstallerAgent::new(provider);

    println!("Installing from URL...");

    let report = installer
        .install_from_url(url, preferred_name, mode)
        .await?;

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

/// 执行 panel1 自升级流程。
fn handle_update_command(
    version: Option<&str>,
    repo: Option<&str>,
    install_dir: Option<&str>,
    no_source_fallback: bool,
) -> Result<()> {
    let installer_url = "https://raw.githubusercontent.com/Gyv12345/panel1/main/install.sh";
    let temp_script = build_temp_installer_path();
    let downloader = pick_downloader()?;

    println!("Downloading installer script...");
    download_installer_script(installer_url, &temp_script, downloader)?;

    println!("Running installer for update...");
    let mut command = Command::new("bash");
    command.arg(&temp_script);

    if let Some(version) = version {
        command.arg("--version").arg(version);
    }
    if let Some(repo) = repo {
        command.arg("--repo").arg(repo);
    }
    if let Some(install_dir) = install_dir {
        command.arg("--install-dir").arg(install_dir);
    }
    if no_source_fallback {
        command.arg("--no-source-fallback");
    }

    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = command
        .status()
        .context("failed to execute installer script for update")?;

    let _ = std::fs::remove_file(&temp_script);

    if !status.success() {
        bail!("update command failed (installer exited with non-zero status)");
    }

    println!("Update completed. Current version:");
    let version_status = Command::new("panel1")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run `panel1 --version` after update")?;

    if !version_status.success() {
        bail!("update succeeded but failed to verify version output");
    }

    Ok(())
}

/// 选择可用下载器（curl 优先，其次 wget）。
fn pick_downloader() -> Result<&'static str> {
    if command_exists("curl") {
        return Ok("curl");
    }
    if command_exists("wget") {
        return Ok("wget");
    }
    bail!("curl or wget is required for update command");
}

/// 检查命令是否存在于当前 PATH。
fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let candidate = dir.join(name);
                candidate.is_file()
            })
        })
        .unwrap_or(false)
}

/// 生成临时安装脚本路径。
fn build_temp_installer_path() -> std::path::PathBuf {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "panel1-install-{}-{now_nanos}.sh",
        std::process::id()
    ))
}

/// 下载安装脚本到本地临时文件。
fn download_installer_script(
    installer_url: &str,
    target_path: &std::path::Path,
    downloader: &str,
) -> Result<()> {
    let status = if downloader == "curl" {
        Command::new("curl")
            .arg("-fsSL")
            .arg(installer_url)
            .arg("-o")
            .arg(target_path)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to run curl for installer script")?
    } else {
        Command::new("wget")
            .arg("-qO")
            .arg(target_path)
            .arg(installer_url)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("failed to run wget for installer script")?
    };

    if !status.success() {
        bail!("failed to download installer script from {}", installer_url);
    }

    Ok(())
}

/// 处理 AI 子命令。
fn handle_ai_command(command: AiCommands) -> Result<()> {
    match command {
        AiCommands::Config {
            profile,
            preset,
            protocol,
            model,
            base_url,
            api_key,
            activate,
        } => {
            let overrides = AiConfigOverrides {
                profile_name: profile,
                preset_name: preset,
                protocol: protocol.map(Into::into),
                model,
                base_url,
                api_key,
                activate,
            };
            configure_ai(overrides)
        }
        AiCommands::Show => show_ai_config(),
        AiCommands::SetModel { model, profile } => set_ai_model(&model, profile.as_deref()),
        AiCommands::Profiles { command } => handle_profile_command(command),
        AiCommands::SeedPresets { overwrite } => seed_presets(overwrite),
    }
}

/// 处理 profile 子命令。
fn handle_profile_command(command: ProfileCommands) -> Result<()> {
    match command {
        ProfileCommands::List => list_profiles(),
        ProfileCommands::Use { name } => switch_active_profile(&name),
        ProfileCommands::Remove { name } => remove_profile(&name),
    }
}

/// 当首次未配置 AI 时，引导用户输入并保存。
fn ensure_ai_config_for_first_time_use() -> Result<()> {
    let mut store = panel_ai::load_ai_store()?;

    if store.active_config().is_some() {
        return Ok(());
    }

    if !can_prompt() {
        return Ok(());
    }

    println!(
        "AI 配置未找到（{}）。可继续安装，但 AI 功能不可用。",
        panel_ai::config_file_path().display()
    );

    if !prompt_yes_no("现在进入 AI 向导并保存配置吗？", true)? {
        return Ok(());
    }

    if store.profiles.is_empty() {
        store = panel_ai::seed_builtin_profiles(false)?;
        println!("已导入内置模型模板，可按需修改。\n");
    }

    let selected = prompt_select_profile(&store)?;
    let profile = store
        .profile(&selected)
        .cloned()
        .with_context(|| format!("Profile '{}' not found", selected))?;

    let finalized = prompt_edit_profile(profile, true)?;
    store.upsert_profile(finalized.clone());
    store.active_profile = finalized.name.clone();
    panel_ai::save_ai_store(&store)?;

    println!("AI 配置已保存，当前激活 profile: {}", finalized.name);
    print_profile_summary(&finalized, true);
    Ok(())
}

/// 创建或更新配置。
fn configure_ai(overrides: AiConfigOverrides) -> Result<()> {
    let mut store = panel_ai::load_ai_store()?;

    if store.profiles.is_empty() {
        let _ = panel_ai::seed_builtin_profiles(false)?;
        store = panel_ai::load_ai_store()?;
    }

    let target_profile_name = overrides
        .profile_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            store
                .active_profile
                .trim()
                .is_empty()
                .then_some(String::new())
                .or_else(|| Some(store.active_profile.clone()))
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "default".to_string());

    let base_profile = if let Some(preset_name) = overrides.preset_name.as_deref() {
        let preset = panel_ai::builtin_model_presets()
            .into_iter()
            .find(|preset| preset.name == preset_name)
            .with_context(|| format!("Preset '{}' not found", preset_name))?;
        let mut profile = preset.to_profile();
        profile.name = target_profile_name.clone();
        profile
    } else if let Some(existing) = store.profile(&target_profile_name) {
        existing.clone()
    } else if let Some(active) = store.active_profile() {
        let mut cloned = active.clone();
        cloned.name = target_profile_name.clone();
        cloned
    } else {
        panel_ai::AiProfile {
            name: target_profile_name.clone(),
            protocol: panel_ai::AiProtocol::Openai,
            model: panel_ai::AiProtocol::Openai.default_model().to_string(),
            base_url: None,
            api_key: None,
            description: Some("created-by-panel1-ai-config".to_string()),
        }
    };

    let interactive = can_prompt();
    let (profile, should_activate) = apply_profile_overrides(base_profile, overrides, interactive)?;

    store.upsert_profile(profile.clone());
    if should_activate
        || profile.name == store.active_profile
        || store.active_profile.trim().is_empty()
    {
        store.active_profile = profile.name.clone();
    }

    panel_ai::save_ai_store(&store)?;

    println!(
        "AI profile '{}' 已保存{}",
        profile.name,
        if store.active_profile == profile.name {
            "（已激活）"
        } else {
            ""
        }
    );
    if let Some(saved) = store.profile(&profile.name) {
        print_profile_summary(saved, false);
    }
    Ok(())
}

/// 应用命令行/交互覆盖项到 profile。
fn apply_profile_overrides(
    mut profile: panel_ai::AiProfile,
    overrides: AiConfigOverrides,
    interactive: bool,
) -> Result<(panel_ai::AiProfile, bool)> {
    if let Some(protocol) = overrides.protocol {
        profile.protocol = protocol;
    } else if interactive {
        profile.protocol = prompt_protocol(profile.protocol)?;
    }

    if let Some(model) = overrides.model {
        profile.model = model;
    } else if interactive {
        profile.model = prompt_line("模型名称", Some(&profile.model))?;
    }

    if let Some(base_url) = overrides.base_url {
        profile.base_url = normalize_optional(base_url);
    } else if interactive {
        let current = profile.base_url.as_deref();
        profile.base_url =
            normalize_optional(prompt_line("Base URL（留空走协议默认地址）", current)?);
    }

    if let Some(api_key) = overrides.api_key {
        profile.api_key = normalize_optional(api_key);
    } else if interactive {
        let current_masked = profile
            .api_key
            .as_deref()
            .map(panel_ai::config::mask_secret)
            .unwrap_or_else(|| "<empty>".to_string());
        if prompt_yes_no(
            &format!("当前 API Key: {current_masked}，需要更新吗？"),
            false,
        )? {
            profile.api_key = normalize_optional(prompt_required("API Key")?);
        }
    }

    profile.name = profile.name.trim().to_string();
    profile.model = profile.model.trim().to_string();
    profile.description = profile
        .description
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if profile.name.is_empty() {
        bail!("Profile name cannot be empty");
    }
    if profile.model.is_empty() {
        bail!("Model cannot be empty");
    }

    Ok((profile, overrides.activate))
}

/// 设置某个 profile 的模型。
fn set_ai_model(model: &str, profile_name: Option<&str>) -> Result<()> {
    let mut store = panel_ai::load_ai_store()?;
    let target = profile_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| store.active_profile.clone());

    if target.trim().is_empty() {
        bail!("No active profile. Run `panel1 ai config` first");
    }

    let profile = store
        .profile_mut(&target)
        .with_context(|| format!("Profile '{}' not found", target))?;

    let trimmed = model.trim();
    if trimmed.is_empty() {
        bail!("Model cannot be empty");
    }

    profile.model = trimmed.to_string();
    panel_ai::save_ai_store(&store)?;

    println!("Profile '{}' model 已更新为: {}", target, trimmed);
    Ok(())
}

/// 展示当前 AI 配置。
fn show_ai_config() -> Result<()> {
    let store = panel_ai::load_ai_store()?;
    let path = panel_ai::config_file_path();

    if store.profiles.is_empty() {
        println!("AI 配置为空: {}", path.display());
        println!("请运行: panel1 ai seed-presets && panel1 ai config");
        return Ok(());
    }

    println!("AI config path: {}", path.display());
    println!("Active profile: {}", store.active_profile);
    println!("Profiles: {}", store.profiles.len());
    println!();

    for profile in &store.profiles {
        print_profile_summary(profile, profile.name == store.active_profile);
    }

    Ok(())
}

/// 列出 profile。
fn list_profiles() -> Result<()> {
    let store = panel_ai::load_ai_store()?;
    if store.profiles.is_empty() {
        println!("暂无 profile。请先运行: panel1 ai seed-presets");
        return Ok(());
    }

    for profile in &store.profiles {
        let marker = if profile.name == store.active_profile {
            "*"
        } else {
            " "
        };
        println!(
            "{marker} {:<24} {:<10} {:<24} {}",
            profile.name,
            profile.protocol.as_str(),
            profile.model,
            profile.base_url.as_deref().unwrap_or("(default)")
        );
    }

    Ok(())
}

/// 切换 active profile。
fn switch_active_profile(name: &str) -> Result<()> {
    let mut store = panel_ai::load_ai_store()?;
    if !store.set_active_profile(name) {
        bail!("Profile '{}' not found", name);
    }
    panel_ai::save_ai_store(&store)?;

    println!("已切换 active profile: {}", name);
    Ok(())
}

/// 删除 profile。
fn remove_profile(name: &str) -> Result<()> {
    let mut store = panel_ai::load_ai_store()?;
    if !store.remove_profile(name) {
        bail!("Profile '{}' not found", name);
    }
    panel_ai::save_ai_store(&store)?;

    println!("已删除 profile: {}", name);
    if store.active_profile.is_empty() {
        println!("当前没有 active profile，请运行 `panel1 ai config` 新建。",);
    } else {
        println!("当前 active profile: {}", store.active_profile);
    }

    Ok(())
}

/// 导入内置模板。
fn seed_presets(overwrite: bool) -> Result<()> {
    let store = panel_ai::seed_builtin_profiles(overwrite)?;
    println!(
        "内置模板导入完成，当前共有 {} 个 profiles，active: {}",
        store.profiles.len(),
        store.active_profile
    );
    Ok(())
}

/// 在终端中选择 profile。
fn prompt_select_profile(store: &panel_ai::AiConfigStore) -> Result<String> {
    if store.profiles.is_empty() {
        bail!("No profiles available");
    }

    println!("可选模型模板：");
    let mut default_index = 1usize;

    for (idx, profile) in store.profiles.iter().enumerate() {
        if profile.name == store.active_profile {
            default_index = idx + 1;
        }
        println!(
            "  {}. {} [{}] {}",
            idx + 1,
            profile.name,
            profile.protocol.as_str(),
            profile.model
        );
    }

    loop {
        let input = prompt_line(
            "选择 profile 编号",
            Some(default_index.to_string().as_str()),
        )?;
        let parsed = input.trim().parse::<usize>();
        if let Ok(number) = parsed {
            if let Some(profile) = store.profiles.get(number.saturating_sub(1)) {
                return Ok(profile.name.clone());
            }
        }
        println!("编号无效，请重试。");
    }
}

/// 交互式编辑 profile。
fn prompt_edit_profile(
    mut profile: panel_ai::AiProfile,
    require_api_key: bool,
) -> Result<panel_ai::AiProfile> {
    profile.protocol = prompt_protocol(profile.protocol)?;
    profile.model = prompt_line("模型名称", Some(&profile.model))?;
    profile.base_url = normalize_optional(prompt_line(
        "Base URL（留空走协议默认地址）",
        profile.base_url.as_deref(),
    )?);

    let api_key = if require_api_key {
        prompt_required("API Key")?
    } else {
        prompt_line("API Key（可留空）", profile.api_key.as_deref())?
    };
    profile.api_key = normalize_optional(api_key);

    Ok(profile)
}

/// 输出 profile 摘要。
fn print_profile_summary(profile: &panel_ai::AiProfile, highlight_active: bool) {
    if highlight_active {
        println!(">>> {} (active)", profile.name);
    } else {
        println!("--- {}", profile.name);
    }

    println!("    Protocol : {}", profile.protocol.as_str());
    println!("    Model    : {}", profile.model);
    println!(
        "    Base URL : {}",
        profile.base_url.as_deref().unwrap_or("(protocol default)")
    );
    println!("    API Key  : {}", profile.masked_api_key());
    if let Some(description) = profile.description.as_deref() {
        println!("    Desc     : {}", description);
    }
    println!();
}

/// 归一化可选字符串。
fn normalize_optional(raw: impl Into<String>) -> Option<String> {
    let value = raw.into();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 判断当前终端是否支持交互输入。
fn can_prompt() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// 提示读取一行文本。
fn prompt_line(prompt: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(default) => {
            print!("{prompt} [{default}]: ");
        }
        None => {
            print!("{prompt}: ");
        }
    }
    io::stdout().flush().context("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read user input")?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.unwrap_or_default().to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// 提示读取必填文本。
fn prompt_required(prompt: &str) -> Result<String> {
    loop {
        let value = prompt_line(prompt, None)?;
        if !value.trim().is_empty() {
            return Ok(value);
        }
        println!("输入不能为空，请重试。");
    }
}

/// 提示读取是/否。
fn prompt_yes_no(prompt: &str, default_yes: bool) -> Result<bool> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{prompt} {suffix}: ");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read user input")?;

    let value = input.trim().to_ascii_lowercase();
    if value.is_empty() {
        return Ok(default_yes);
    }

    Ok(matches!(value.as_str(), "y" | "yes"))
}

/// 提示读取协议类型。
fn prompt_protocol(default: panel_ai::AiProtocol) -> Result<panel_ai::AiProtocol> {
    loop {
        let value = prompt_line("协议类型（openai/anthropic）", Some(default.as_str()))?;
        if let Some(protocol) = panel_ai::AiProtocol::parse(&value) {
            return Ok(protocol);
        }
        println!("无效协议，请输入 openai 或 anthropic。");
    }
}

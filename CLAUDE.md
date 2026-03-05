# CLAUDE.md - Panel1 开发指南

> 本文件为 AI 助手（如 Claude）提供项目上下文，帮助更好地理解和操作此代码库。

## 项目概述

Panel1 是一个 Linux 服务器管理面板，采用 Rust Workspace 架构。提供 TUI 终端界面、AI 智能助手和混合模式服务管理。

**技术栈**：Rust 2021 edition | Tokio async | ratatui (TUI) | clap (CLI) | genai (AI)

## 常用命令

```bash
# 开发
cargo run                          # 运行（默认 TUI 界面）
cargo build --release              # 构建发布版本

# 质量检查（CI 必须通过）
cargo check --all-targets          # 编译检查
cargo test --all                   # 运行所有测试
cargo fmt --all -- --check         # 格式化检查
cargo clippy --all-targets -- -W clippy::all  # Lint 检查

# 发布构建
./build-release.sh                 # 打包 tar.gz（输出到 dist/）
```

## Workspace 架构

```
crates/
├── panel-core/      # 核心库：系统信息、进程、服务、网络（sysinfo）
├── panel-docker/    # Docker 管理（bollard）
├── panel-service/   # 混合服务管理：systemd / Panel1 托管 / Docker
├── panel-ai/        # AI Agent：安装助手 + 运维顾问（genai）
├── panel-tui/       # TUI 界面（ratatui + crossterm）
└── panel-cli/       # CLI 入口（clap）
```

**依赖关系**：
- `panel-cli` 依赖所有其他 crate
- `panel-tui` 依赖 `panel-core`, `panel-service`, `panel-ai`
- `panel-service` 依赖 `panel-core`, `panel-docker`
- `panel-ai` 依赖 `panel-core`

## 代码风格

- Rust 2021 edition
- 使用 `[workspace.dependencies]` 管理共享依赖
- 错误处理：`anyhow` (应用层) + `thiserror` (库层)
- 日志：`tracing` + `tracing-subscriber`

## AI 模块说明

`panel-ai` 使用 **genai** 库（非 claude-agent），支持：

- 自定义 AI 网关配置
- 多后端支持（OpenAI / Ollama）
- 两个 Agent：
  - `InstallerAgent`：软件安装指导
  - `AdvisorAgent`：运维诊断建议
- 内置工具：`ShellTool`、`DiagnosticTool`

## CI 流程

GitHub Actions CI 在每次 push/PR 到 master 或 main 时运行：

1. `cargo check --all-targets`
2. `cargo test --all`
3. `cargo fmt --all -- --check`
4. `cargo clippy --all-targets -- -W clippy::all`

**提交前请确保本地通过以上所有检查。**

## 文件约定

- 每个 crate 有自己的 `Cargo.toml`，使用 `workspace = true` 继承版本和 edition
- 入口点：`crates/panel-cli/src/main.rs`
- TUI 状态机：`crates/panel-tui/src/app.rs`

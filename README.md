# Panel1 - Linux 服务器管理面板

一个现代化的 Linux 服务器管理面板，提供极简 TUI（监控 + AI 安装 Agent）。

## 特性

- **🖥️ TUI 终端界面** - 交互式命令行操作体验
- **🤖 AI 安装 Agent** - 输入 URL 自动安装工具服务（失败自动重试）
- **📦 单二进制部署** - 无需依赖，开箱即用

## 安装

### 一键安装（Linux，推荐）

```bash
curl -fsSL https://raw.githubusercontent.com/Gyv12345/panel1/main/install.sh | bash
```

指定版本安装：

```bash
curl -fsSL https://raw.githubusercontent.com/Gyv12345/panel1/main/install.sh | bash -s -- --version v0.1.0
```

### 从 GitHub Releases 下载

```bash
# 下载最新版本（Linux x86_64）
wget https://github.com/Gyv12345/panel1/releases/latest/download/panel1-0.1.0-x86_64-unknown-linux-gnu.tar.gz

# 解压
tar -xzf panel1-0.1.0-x86_64-unknown-linux-gnu.tar.gz

# 安装
sudo cp panel1-0.1.0-x86_64-unknown-linux-gnu/bin/panel1 /usr/local/bin/
```

### 从源码编译

```bash
git clone https://github.com/Gyv12345/panel1.git
cd panel1
cargo build --release
sudo cp target/release/panel1 /usr/local/bin/
```

## 使用方法

```bash
# 启动 TUI 界面（默认）
panel1

# 启动极简 TUI（两页：监控 / AI 安装）
panel1 tui

# 查看系统状态
panel1 status

# 通过 URL 安装工具（Agent 模式）
panel1 install https://example.com/tool.tar.gz
panel1 install https://example.com/my-tool --name my-tool
panel1 install https://example.com/tool.tar.gz --verbose
```

## TUI 界面

| 快捷键 | 功能 |
|--------|------|
| `1` | 服务器监控 |
| `2` | AI 安装 Agent |
| `Tab` / `↑↓` | AI 页切换输入项 |
| `Enter` | 提交 URL 并自动安装 |
| `?` | 帮助 |
| `q` | 退出 |

## 安装说明

`panel1 install <url>` 会自动执行：
1. 下载文件
2. 识别归档并尝试解压
3. 自动重试和基础自修复
4. 写入本地服务目录并设置可执行权限

## Linux 兼容性

- 发行包优先使用 `musl` 静态构建（兼容性更高），并回退 `gnu` 构建。
- 当预编译包不可用时，安装脚本可回退到 `cargo` 源码构建。

## 项目结构

```
crates/
├── panel-core/      # 核心库（系统信息、服务管理）
├── panel-service/   # URL 安装与服务托管
├── panel-ai/        # 安装 Agent
├── panel-tui/       # TUI 终端界面
└── panel-cli/       # CLI 入口
```

## 开发

```bash
# 开发模式运行
cargo run

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化
cargo fmt
```

## License

MIT

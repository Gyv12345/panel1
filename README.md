# Panel1 - Linux 服务器管理面板

一个现代化的 Linux 服务器管理面板，提供 TUI 终端界面、AI 智能助手和混合模式服务管理。

## 特性

- **🖥️ TUI 终端界面** - 交互式命令行操作体验
- **🤖 AI 智能助手** - 安装助手 + 运维顾问（支持 OpenAI / Ollama）
- **⚙️ 混合服务管理** - 支持 systemd / Panel1 托管 / Docker 三种模式
- **📦 单二进制部署** - 无需依赖，开箱即用

## 安装

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

# 直接启动安装向导
panel1 tui --wizard

# 直接启动 AI 对话
panel1 tui --chat

# 查看系统状态
panel1 status

# 服务管理
panel1 service list
panel1 service start nginx
panel1 service stop nginx

# AI 功能
panel1 ai install redis      # AI 安装建议
panel1 ai diagnose           # 系统诊断
panel1 ai optimize           # 性能优化建议
panel1 ai security           # 安全检查
panel1 ai ask "如何优化内存"  # 自由问答

# 安装服务
panel1 install redis --mode panel1
panel1 install nginx --mode systemd
```

## TUI 界面

| 快捷键 | 功能 |
|--------|------|
| `1` | 仪表盘 |
| `2` | 服务管理 |
| `3` | 安装向导 |
| `4` | AI 对话 |
| `?` | 帮助 |
| `q` | 退出 |

## AI 配置

支持 OpenAI 和 Ollama（本地模型）：

```bash
# 使用 OpenAI
export OPENAI_API_KEY="your-api-key"
panel1 ai ask "问题"

# 使用 Ollama（本地）
ollama serve &
ollama pull llama3
panel1 ai ask "问题"  # 自动使用 Ollama
```

## 项目结构

```
crates/
├── panel-core/      # 核心库（系统信息、服务管理）
├── panel-docker/    # Docker 管理
├── panel-service/   # 混合模式服务管理
├── panel-ai/        # AI Agent（LLM、诊断工具）
├── panel-tui/       # TUI 终端界面
└── panel-cli/       # CLI 入口
```

## 支持的服务

| 服务 | systemd | Panel1 | Docker |
|------|---------|--------|--------|
| Redis | ✅ | ✅ | ✅ |
| Nginx | ✅ | ✅ | ✅ |
| PostgreSQL | ✅ | ✅ | ✅ |
| Elasticsearch | ✅ | ✅ | ✅ |

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

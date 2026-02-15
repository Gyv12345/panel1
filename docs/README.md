# Panel1 - Linux 服务器管理面板

一个使用 Rust + React 开发的 Linux 服务器管理面板，类似宝塔面板、1Panel，采用前后端嵌入架构，单二进制部署。

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端框架 | Axum + Tokio |
| 前端框架 | React 18 + TypeScript |
| UI 组件 | Tailwind CSS |
| 数据库 | SQLite (sqlx) |
| Docker 交互 | bollard |
| 前端嵌入 | rust-embed |
| 认证 | JWT (jsonwebtoken) |

## 项目结构

```
panel1/
├── Cargo.toml                    # Workspace 配置
├── crates/
│   ├── panel-core/              # 核心库：Linux 系统交互
│   │   └── src/
│   │       ├── system.rs        # 系统信息（CPU/内存/磁盘）
│   │       ├── process.rs       # 进程管理
│   │       ├── service.rs       # systemd 服务管理
│   │       ├── network.rs       # 网络配置
│   │       └── lib.rs
│   │
│   ├── panel-docker/            # Docker 管理
│   │   └── src/
│   │       ├── container.rs     # 容器操作
│   │       ├── image.rs         # 镜像管理
│   │       ├── compose.rs       # Docker Compose
│   │       └── lib.rs
│   │
│   ├── panel-web/               # Web 服务 + 前端嵌入
│   │   ├── src/
│   │   │   ├── main.rs          # 入口
│   │   │   ├── routes/          # API 路由
│   │   │   ├── services/        # 业务逻辑层
│   │   │   ├── models/          # 数据模型
│   │   │   └── middleware/      # JWT 认证中间件
│   │   ├── frontend/            # React 前端
│   │   ├── migrations/          # 数据库迁移
│   │   └── Cargo.toml
│   │
│   └── panel-cli/               # 命令行工具
│
└── docs/                        # 文档
```

## 功能模块

### 1. 系统监控
- CPU 使用率和核心信息
- 内存使用情况
- 磁盘空间监控
- 网络接口统计
- 进程列表和管理

### 2. 服务管理
- systemd 服务列表
- 服务启停控制
- 服务状态监控

### 3. Docker 管理
- 容器列表/启停/删除
- 镜像列表/拉取/删除
- 容器日志查看
- Docker Compose 支持

### 4. 文件管理
- 目录浏览
- 文件读写
- 创建/删除文件和目录

### 5. 网站管理
- Nginx 配置管理
- 域名绑定
- SSL 证书配置

### 6. AI 应用管理
- 一键安装 n8n（工作流自动化）
- 一键安装 Open WebUI（LLM 界面）
- 一键安装 Qdrant（向量数据库）
- 一键安装 Dify（LLM 应用平台）

## API 端点

### 认证
```
POST   /api/auth/login          # 登录
POST   /api/auth/logout         # 登出
GET    /api/auth/me             # 获取当前用户
PUT    /api/auth/password       # 修改密码
```

### 系统监控
```
GET    /api/system/info         # 系统基本信息
GET    /api/system/stats        # 实时数据
GET    /api/system/processes    # 进程列表
GET    /api/system/network      # 网络统计
GET    /api/system/services     # 服务列表
```

### Docker
```
GET    /api/containers          # 容器列表
POST   /api/containers/:id/start
POST   /api/containers/:id/stop
POST   /api/containers/:id/restart
DELETE /api/containers/:id
GET    /api/containers/:id/logs
GET    /api/images              # 镜像列表
POST   /api/images/pull         # 拉取镜像
DELETE /api/images/:id
```

### 文件管理
```
GET    /api/files               # 列出目录
GET    /api/files/content       # 读取文件
POST   /api/files               # 创建
PUT    /api/files               # 修改
DELETE /api/files               # 删除
```

### 网站管理
```
GET    /api/websites            # 网站列表
POST   /api/websites            # 创建网站
PUT    /api/websites/:id        # 修改
DELETE /api/websites/:id        # 删除
POST   /api/websites/:id/ssl    # 配置 SSL
```

### AI 应用
```
GET    /api/apps                # 已安装应用
GET    /api/apps/templates      # 应用模板
POST   /api/apps/install        # 安装应用
POST   /api/apps/:id/start
POST   /api/apps/:id/stop
DELETE /api/apps/:id
```

## 数据库模型

### 用户表 (users)
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_login DATETIME
);
```

### 网站配置 (websites)
```sql
CREATE TABLE websites (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT NOT NULL UNIQUE,
    root_path TEXT NOT NULL,
    port INTEGER DEFAULT 80,
    ssl_enabled INTEGER DEFAULT 0,
    ssl_cert_path TEXT,
    ssl_key_path TEXT,
    nginx_config TEXT,
    status TEXT DEFAULT 'stopped',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 应用配置 (apps)
```sql
CREATE TABLE apps (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    app_type TEXT NOT NULL,
    docker_compose_path TEXT,
    port INTEGER,
    status TEXT DEFAULT 'stopped',
    config JSON,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## 构建和运行

### 1. 构建后端
```bash
cargo build --release
```

### 2. 安装前端依赖
```bash
cd crates/panel-web/frontend
pnpm install
```

### 3. 构建前端
```bash
pnpm build
```

### 4. 运行
```bash
# 设置数据目录
export PANEL_DATA_DIR=/opt/panel/data

# 创建数据目录
mkdir -p /opt/panel/data

# 运行服务
./target/release/panel1
```

### 5. 访问
打开浏览器访问 `http://localhost:3000`

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| PANEL_DATA_DIR | 数据存储目录 | /opt/panel/data |
| PANEL_CONFIG_DIR | 配置文件目录 | /opt/panel/config |
| PANEL_FILE_ROOT | 文件管理根目录 | / |

## 数据目录结构

```
/opt/panel/
├── data/
│   ├── panel.db              # SQLite 数据库
│   └── apps/                 # AI 应用数据
│       ├── n8n/
│       ├── openwebui/
│       └── qdrant/
├── config/
│   └── nginx/                # Nginx 配置
└── logs/
    └── panel.log
```

## 开发指南

### 运行开发服务器
```bash
# 后端
cargo run -p panel-web

# 前端开发
cd crates/panel-web/frontend
pnpm dev
```

### 数据库迁移
```bash
# 添加新迁移
# 在 crates/panel-web/migrations/ 目录下创建新的 SQL 文件
```

## 安全措施

1. **密码**: bcrypt 哈希 (cost = 12)
2. **API 保护**: `/api/*` 需要 JWT 认证
3. **操作审计**: 所有写操作记录日志
4. **静态资源**: 无需认证

## 内置应用模板

| 应用 | 说明 | 默认端口 |
|------|------|---------|
| n8n | 工作流自动化 | 5678 |
| Open WebUI | LLM 界面 | 3000 |
| Dify | LLM 应用平台 | 8080 |
| Qdrant | 向量数据库 | 6333 |

## License

MIT

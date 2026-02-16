#!/bin/bash
# Panel1 Linux 交叉编译脚本
# 在 macOS 上为 Ubuntu Linux x86_64 构建

set -e

VERSION=${VERSION:-0.1.0}
TARGET="x86_64-unknown-linux-gnu"
DIST_DIR="dist/panel1-${VERSION}-linux-x86_64"

echo "=== Building Panel1 v${VERSION} for Linux x86_64 ==="

# 检查 Docker 是否运行
if ! docker info &>/dev/null; then
    echo "Error: Docker is not running. Please start Docker first."
    exit 1
fi

# 检查并安装 cross
if ! command -v cross &>/dev/null; then
    echo "Installing cross tool..."
    cargo install cross --git https://github.com/cross-rs/cross
fi

# 添加 Rust 目标
echo "Adding Rust target ${TARGET}..."
rustup target add ${TARGET} 2>/dev/null || true

# 构建 release 版本
echo "Building release binary for ${TARGET}..."
cross build --release --target ${TARGET}

# 创建发布目录
echo "Creating distribution directory..."
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/bin"
mkdir -p "${DIST_DIR}/data"
mkdir -p "${DIST_DIR}/services"

# 复制二进制文件
echo "Copying binary..."
cp target/${TARGET}/release/panel1 "${DIST_DIR}/bin/"
chmod +x "${DIST_DIR}/bin/"*

# 复制文档
echo "Copying documentation..."
cp README.md "${DIST_DIR}/" 2>/dev/null || echo "# Panel1 - Linux Server Management Panel" > "${DIST_DIR}/README.md"
cp LICENSE "${DIST_DIR}/" 2>/dev/null || echo "MIT License" > "${DIST_DIR}/LICENSE"

# 创建安装脚本
cat > "${DIST_DIR}/install.sh" << 'INSTALL_EOF'
#!/bin/bash
set -e

INSTALL_DIR="${INSTALL_DIR:-/opt/panel}"
DATA_DIR="${DATA_DIR:-/opt/panel/data}"
SERVICE_DIR="${SERVICE_DIR:-/opt/panel/services}"
CACHE_DIR="${CACHE_DIR:-/var/cache/panel1}"

echo "Installing Panel1 to ${INSTALL_DIR}..."

# 创建目录
sudo mkdir -p "${INSTALL_DIR}/bin"
sudo mkdir -p "${DATA_DIR}"
sudo mkdir -p "${SERVICE_DIR}"
sudo mkdir -p "${CACHE_DIR}/registry"
sudo mkdir -p "${CACHE_DIR}/downloads"

# 复制文件
sudo cp bin/panel1 "${INSTALL_DIR}/bin/"
sudo chmod +x "${INSTALL_DIR}/bin/"*

# 创建符号链接
sudo ln -sf "${INSTALL_DIR}/bin/panel1" /usr/local/bin/panel1

# 创建 systemd 服务（可选）
read -p "Create systemd service for auto-start? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo tee /etc/systemd/system/panel1.service > /dev/null << 'SERVICE_EOF'
[Unit]
Description=Panel1 Server Management Panel
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/panel1
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
SERVICE_EOF
    sudo systemctl daemon-reload
    sudo systemctl enable panel1
    echo "Systemd service created and enabled."
fi

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Usage:"
echo "  panel1              # Start TUI interface"
echo "  panel1 tui          # Start TUI interface"
echo "  panel1 tui --wizard # Start installation wizard"
echo "  panel1 tui --chat   # Start AI chat"
echo "  panel1 status       # Show system status"
echo "  panel1 service list # List all services"
echo "  panel1 ai diagnose  # Run AI diagnostics"
echo "  panel1 --help       # Show all commands"
INSTALL_EOF
chmod +x "${DIST_DIR}/install.sh"

# 创建卸载脚本
cat > "${DIST_DIR}/uninstall.sh" << 'UNINSTALL_EOF'
#!/bin/bash
set -e

INSTALL_DIR="${INSTALL_DIR:-/opt/panel}"
CACHE_DIR="${CACHE_DIR:-/var/cache/panel1}"

echo "Uninstalling Panel1..."

# 停止并禁用服务
if systemctl is-active panel1 &>/dev/null; then
    sudo systemctl stop panel1
fi
if systemctl is-enabled panel1 &>/dev/null; then
    sudo systemctl disable panel1
fi

# 删除 systemd 服务文件
sudo rm -f /etc/systemd/system/panel1.service
sudo systemctl daemon-reload 2>/dev/null || true

# 删除符号链接
sudo rm -f /usr/local/bin/panel1

# 删除安装目录（可选）
read -p "Remove installation directory ${INSTALL_DIR}? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo rm -rf "${INSTALL_DIR}"
fi

# 删除缓存目录（可选）
read -p "Remove cache directory ${CACHE_DIR}? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo rm -rf "${CACHE_DIR}"
fi

echo "Uninstallation complete!"
UNINSTALL_EOF
chmod +x "${DIST_DIR}/uninstall.sh"

# 打包
echo "Creating archive..."
cd dist
COPYFILE_DISABLE=1 tar -czvf "panel1-${VERSION}-linux-x86_64.tar.gz" "panel1-${VERSION}-linux-x86_64"

echo ""
echo "=== Build Complete ==="
echo "Distribution package: dist/panel1-${VERSION}-linux-x86_64.tar.gz"
echo ""
echo "To install on Ubuntu server:"
echo "  # Copy the tar.gz to your server"
echo "  tar -xzf panel1-${VERSION}-linux-x86_64.tar.gz"
echo "  cd panel1-${VERSION}-linux-x86_64"
echo "  sudo ./install.sh"

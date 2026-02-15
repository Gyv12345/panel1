#!/bin/bash
# Panel1 发布打包脚本

set -e

VERSION=${VERSION:-0.1.0}
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
DIST_DIR="dist/panel1-${VERSION}-${OS}-${ARCH}"

echo "=== Building Panel1 v${VERSION} for ${OS}-${ARCH} ==="

# 构建 release 版本
echo "Building release binary..."
cargo build --release

# 创建发布目录
echo "Creating distribution directory..."
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/bin"
mkdir -p "${DIST_DIR}/data"
mkdir -p "${DIST_DIR}/services"

# 复制二进制文件
echo "Copying binary..."
cp target/release/panel1 "${DIST_DIR}/bin/"
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

echo "Installing Panel1 to ${INSTALL_DIR}..."

# 创建目录
sudo mkdir -p "${INSTALL_DIR}/bin"
sudo mkdir -p "${DATA_DIR}"

# 复制文件
sudo cp bin/panel1 "${INSTALL_DIR}/bin/"
sudo chmod +x "${INSTALL_DIR}/bin/"*

# 创建符号链接
sudo ln -sf "${INSTALL_DIR}/bin/panel1" /usr/local/bin/panel1

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

echo "Uninstalling Panel1..."

# 删除符号链接
sudo rm -f /usr/local/bin/panel1

# 删除安装目录（可选）
read -p "Remove installation directory ${INSTALL_DIR}? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo rm -rf "${INSTALL_DIR}"
fi

echo "Uninstallation complete!"
UNINSTALL_EOF
chmod +x "${DIST_DIR}/uninstall.sh"

# 打包
echo "Creating archive..."
cd dist
tar -czvf "panel1-${VERSION}-${OS}-${ARCH}.tar.gz" "panel1-${VERSION}-${OS}-${ARCH}"

echo ""
echo "=== Build Complete ==="
echo "Distribution package: dist/panel1-${VERSION}-${OS}-${ARCH}.tar.gz"
echo ""
echo "To install on a server:"
echo "  tar -xzf panel1-${VERSION}-${OS}-${ARCH}.tar.gz"
echo "  cd panel1-${VERSION}-${OS}-${ARCH}"
echo "  sudo ./install.sh"

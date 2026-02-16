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

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

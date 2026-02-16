#!/bin/bash
# Panel1 Linux Docker 构建脚本
# 使用 Docker 容器直接编译，不需要 cross 工具

set -e

VERSION=${VERSION:-0.1.0}
DIST_DIR="dist/panel1-${VERSION}-linux-x86_64"
IMAGE_NAME="panel1-builder"

echo "=== Building Panel1 v${VERSION} for Linux x86_64 (Docker) ==="

# 检查 Docker 是否运行
if ! docker info &>/dev/null; then
    echo "Error: Docker is not running. Please start Docker first."
    exit 1
fi

# 构建 Docker 镜像
echo "Building Docker image..."
docker build -t ${IMAGE_NAME} -f Dockerfile.build .

# 创建发布目录
echo "Creating distribution directory..."
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/bin"

# 从 Docker 镜像中提取二进制文件
echo "Extracting binary from Docker image..."
CONTAINER_ID=$(docker create ${IMAGE_NAME})
docker cp ${CONTAINER_ID}:/build/target/release/panel1 "${DIST_DIR}/bin/"
docker rm ${CONTAINER_ID}

chmod +x "${DIST_DIR}/bin/"*

# 复制文档
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

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Usage: panel1 [command]"
echo "Run 'panel1 --help' for more information."
INSTALL_EOF
chmod +x "${DIST_DIR}/install.sh"

# 创建卸载脚本
cat > "${DIST_DIR}/uninstall.sh" << 'UNINSTALL_EOF'
#!/bin/bash
set -e

INSTALL_DIR="${INSTALL_DIR:-/opt/panel}"
sudo rm -f /usr/local/bin/panel1
read -p "Remove ${INSTALL_DIR}? [y/N] " -n 1 -r
echo
[[ $REPLY =~ ^[Yy]$ ]] && sudo rm -rf "${INSTALL_DIR}"
echo "Uninstallation complete!"
UNINSTALL_EOF
chmod +x "${DIST_DIR}/uninstall.sh"

# 打包
echo "Creating archive..."
cd dist
COPYFILE_DISABLE=1 tar -czvf "panel1-${VERSION}-linux-x86_64.tar.gz" "panel1-${VERSION}-linux-x86_64"

# 清理 Docker 镜像（可选）
echo ""
read -p "Remove builder Docker image? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    docker rmi ${IMAGE_NAME}
fi

echo ""
echo "=== Build Complete ==="
echo "Distribution package: dist/panel1-${VERSION}-linux-x86_64.tar.gz"

#!/bin/bash

# 构建脚本

set -e  # 出错时退出

echo "实时同声翻译应用构建脚本"
echo "=========================="

# 检查Rust是否已安装
if ! command -v rustc &> /dev/null; then
    echo "错误: Rust未安装"
    echo "请先安装Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✓ Rust已安装: $(rustc --version)"

# 检查Cargo是否已安装
if ! command -v cargo &> /dev/null; then
    echo "错误: Cargo未安装"
    exit 1
fi

echo "✓ Cargo已安装: $(cargo --version)"

# 更新依赖
echo -e "\n1. 更新依赖..."
cargo fetch

# 构建项目
echo -e "\n2. 构建项目..."
cargo build

# 运行测试
echo -e "\n3. 运行测试..."
cargo test

# 构建演示
echo -e "\n4. 构建集成演示..."
# 创建一个简单的main函数来运行演示
cat > src/bin/integration_demo.rs << 'EOF'
include!("../../integration_demo.rs")
EOF

cargo build --bin integration_demo

echo -e "\n5. 构建完成!"
echo "=================="
echo "项目已成功构建。您可以："
echo "- 运行演示: cargo run --bin integration_demo"
echo "- 运行测试: cargo test"
echo "- 运行主应用: cargo run"
echo ""
echo "要使用真实模型，请："
echo "1. 下载模型文件: ./download_models.sh"
echo "2. 更新模型路径"
echo "3. 重新构建项目"
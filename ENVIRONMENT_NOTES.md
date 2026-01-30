# 环境配置说明

## 开发环境设置

### Rust 工具链

确保安装了 Rust 和 Cargo：

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 依赖项

根据目标平台安装相应依赖：

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev libasound2-dev
```

#### CentOS/RHEL/Fedora
```bash
sudo dnf install gcc pkgconfig openssl-devel alsa-lib-devel
# 或对于较老版本使用 yum
```

#### macOS
```bash
# 安装 Xcode 命令行工具
xcode-select --install

# 如果使用 Homebrew
brew install pkg-config openssl
```

#### Windows
```powershell
# 使用 vcpkg 或者安装相应的 MSYS2 包
# 确保安装了 Visual Studio Build Tools
```

## 项目结构

```
real_trans/
├── Cargo.toml          # 项目配置和依赖
├── Cargo.lock          # 依赖锁定版本
├── README.md           # 项目说明
├── TROUBLESHOOTING.md  # 故障排除指南
├── ENVIRONMENT_NOTES.md # 本文件
├── src/                # 源代码目录
│   ├── lib.rs          # 库入口
│   ├── main.rs         # 可执行文件入口
│   ├── audio_types.rs  # 音频类型定义
│   ├── audio_switchboard.rs  # 音频交换板
│   ├── virtual_audio_manager.rs  # 虚拟音频管理器
│   ├── bidirectional_translator.rs  # 双向翻译器
│   ├── io/             # 音频 I/O 模块
│   │   ├── mod.rs
│   │   ├── audio_device.rs
│   │   ├── virtual_audio_device.rs
│   │   └── audio_capture.rs
│   └── engine/         # 翻译引擎模块
│       ├── mod.rs
│       ├── asr.rs
│       ├── mt.rs
│       ├── tts.rs
│       └── vad.rs
├── examples/           # 示例代码
│   ├── simple_demo.rs
│   ├── full_demo.rs
│   ├── list_audio_devices.rs
│   └── module_tests/   # 模块测试示例
│       ├── input_test.rs
│       ├── output_test.rs
│       ├── input_with_translation_test.rs
│       └── full_integration_test.rs
├── models/             # 模型文件目录 (需手动创建和填充)
└── target/             # 编译输出目录 (由 Cargo 自动生成)
```

## 音频参数配置

当前使用的音频参数：

```rust
pub const SAMPLE_RATE: u32 = 16000;        // 采样率：16kHz
pub const CHANNELS: u16 = 1;               // 声道数：单声道
pub const BITS_PER_SAMPLE: u16 = 16;       // 位深度：16位
pub const FRAME_SIZE_MS: u32 = 20;         // 帧大小：20ms
pub const SAMPLES_PER_FRAME: usize = 320;  // 每帧样本数 (16000 * 20 / 1000)
pub type AudioSample = i16;                // 音频采样点类型
```

## 模型文件要求

项目需要以下模型文件（放置在 `./models/` 目录）：

- `whisper-tiny.bin` - ASR (自动语音识别) 模型
- `qwen2.5-0.5b.bin` - MT (机器翻译) 模型
- 其他 TTS (文本转语音) 模型文件

### 创建模型目录
```bash
mkdir -p models
# 然后将模型文件复制到此目录
```

## 开发工作流

### 1. 本地开发
```bash
# 检查代码语法
cargo check

# 构建项目
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example simple_demo
```

### 2. 代码格式化和清理
```bash
# 格式化代码
cargo fmt

# 检查未使用的代码
cargo clippy

# 自动修复一些问题
cargo clippy --fix
```

### 3. 性能分析
```bash
# 构建发布版本
cargo build --release

# 性能分析 (需要安装 cargo-profiler)
cargo install flamegraph
cargo flamegraph --example simple_demo
```

## 调试配置

### 日志级别设置
```bash
# 设置详细日志输出
export RUST_LOG=debug
# 或者
RUST_LOG=trace cargo run --example simple_demo
```

### IDE 配置建议

#### VS Code
推荐安装以下扩展：
- rust-analyzer
- crates
- Error Lens

VS Code 配置 (.vscode/settings.json)：
```json
{
  "rust-analyzer.checkOnSave.command": "check",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true
}
```

## 测试策略

### 单元测试
```bash
# 运行所有单元测试
cargo test

# 运行特定测试
cargo test test_function_name

# 运行不包括集成测试
cargo test --lib
```

### 集成测试
```bash
# 运行示例作为集成测试
cargo run --example simple_demo
cargo run --example full_demo
```

### 模块测试
```bash
# 运行模块级别的测试示例
cargo run --example module_tests/input_test
cargo run --example module_tests/output_test
cargo run --example module_tests/input_with_translation_test
cargo run --example module_tests/full_integration_test
```

## 持续集成配置

如果项目需要 CI/CD，可以参考以下 `.github/workflows/rust.yml` 模板：

```yaml
name: Rust

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:

    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
    - name: Install dependencies
      run: sudo apt-get install -y libasound2-dev
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
```

## 部署准备

### 交叉编译
```bash
# 添加目标架构 (例如 ARM)
rustup target add armv7-unknown-linux-gnueabihf

# 构建到特定目标
cargo build --target armv7-unknown-linux-gnueabihf --release
```

### 发布版本构建
```bash
# 构建优化的发布版本
cargo build --release

# 构建并安装到本地
cargo install --path .
```
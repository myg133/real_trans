# 实时同声翻译应用 (Real-time Speech Translation, RST)

一个专注于本地化、低延时、跨平台的实时翻译系统，使用Rust编写。

## 架构概述

系统采用**生产者-消费者（Producer-Consumer）**模型，通过高性能环形缓冲区处理音频流，确保 UI 线程与推理线程解耦。

### 模块组成

- **Core**: 核心数据结构（环形缓冲区、音频类型定义）
- **IO**: 音频输入输出（音频设备、音频捕获）
- **Engine**: 核心引擎
  - ASR: 自动语音识别（使用whisper-rs）
  - MT: 机器翻译（使用llm库）
  - VAD: 语音活动检测（使用Silero VAD）
  - TTS: 文本转语音（可选功能）
  - Translation Pipeline: 翻译流水线协调器
  - Model Loader: 模型加载器
- **UI**: 用户界面（待实现）

## 依赖库

- `whisper-rs`: 用于ASR（语音转文本）
- `llm`: 用于MT（机器翻译）
- `tokio`: 异步运行时
- `serde`: 数据序列化
- `anyhow`: 错误处理
- 其他辅助库

## 模型要求

### ASR模型
- 推荐使用Whisper系列模型 (tiny, base, small等)
- 量化版本以提高性能 (ggml格式)
- 模型文件应放置在 `./models/` 目录下

### MT模型
- 推荐使用Qwen2.5-0.5B或Llama-3.2-1B的GGUF量化版本
- 模型文件应放置在 `./models/` 目录下

### VAD模型
- 使用Silero VAD模型进行语音活动检测

## 性能目标

- 首字延时: < 800ms
- 内存占用: < 1.2GB
- CPU占用: < 20% (M1/M2或Intel i7移动端基准测试)
- 断句准确率: > 90%

## 使用方法

### 快速开始

```bash
# 1. 安装依赖
cargo build

# 2. 运行集成演示
cargo run --bin integration_demo

# 3. 运行测试
cargo test
```

### 模型安装

```bash
# 下载模型文件
./download_models.sh

# 或手动下载模型到 ./models/ 目录
```

### 运行完整应用

```bash
# 1. 下载模型文件
./download_models.sh

# 2. 更新模型路径为实际的模型文件路径

# 3. 运行应用
cargo run
```

## 开发状态

- [x] 基础架构搭建
- [x] 环形缓冲区实现
- [x] 音频输入输出模块
- [x] ASR模块框架
- [x] MT模块框架
- [x] VAD模块框架
- [x] 翻译流水线
- [x] 模型加载器
- [x] 集成演示
- [ ] 真实模型集成 (进行中)
- [ ] 性能优化
- [ ] UI界面开发

## 模型集成说明

已完成模型加载器的框架实现：

1. **ASR模块**: 使用whisper-rs库集成Whisper模型
2. **MT模块**: 使用llm库集成Qwen或Llama模型
3. **VAD模块**: 集成Silero VAD进行语音活动检测
4. **Model Loader**: 统一的模型加载接口

### 模型加载流程

1. 检查模型文件是否存在
2. 加载模型到内存
3. 初始化推理引擎
4. 验证模型兼容性

## 集成演示

运行集成演示以查看所有组件如何协同工作：

```bash
# 编译并运行集成演示
rustc integration_demo.rs --extern real_trans=target/debug/deps/libreal_trans-*.rlib
# 或者通过cargo运行
```

## 构建说明

```bash
# 安装依赖
cargo build

# 运行测试
cargo test

# 运行应用
cargo run
```

## 模型文件下载

使用 `download_models.sh` 脚本下载所需的模型文件：

```bash
# 使脚本可执行
chmod +x download_models.sh

# 运行下载脚本
./download_models.sh
```

该脚本将提供模型下载的指导和链接。
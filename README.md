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
- **Bidirectional Translator**: 双向翻译器 - 实现用户语言和对方语言之间的双向实时翻译
- **Virtual Audio Manager**: 虚拟音频设备管理器 - 管理虚拟音频输入和输出设备
- **Tests**: 测试模块 - 包含音频模拟测试框架
- **UI**: 用户界面（待实现）

## 功能特性

### 双向实时翻译
- **用户语言到对方语言**：用户说话时实时翻译成对方语言
- **对方语言到用户语言**：对方说话时实时翻译成用户语言
- **动态语言切换**：支持会议中动态更改语言对

### 虚拟音频设备
- **虚拟输入设备**：用于捕获用户语音
- **虚拟输出设备**：用于播放翻译后的语音
- **在线会议集成**：可在Zoom、Teams等会议软件中使用

### 实时处理
- **低延迟**：端到端延迟小于800ms
- **流式处理**：支持实时语音流处理
- **高保真**：保持原始音频质量

### 音频模拟测试
- **文件驱动测试**：基于文件的音频输入/输出模拟
- **目录监控**：自动监控和处理音频文件
- **结果验证**：自动验证翻译结果

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

# 2. 运行完整演示
cargo run --bin full_demo

# 3. 运行简单演示
cargo run --bin simple_demo

# 4. 运行集成演示
cargo run --bin integration_demo

# 5. 运行会议翻译器演示
cargo run --bin conference_translator_demo

# 6. 运行测试
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

## 使用场景

### 在线会议
1. 程序启动后，系统创建虚拟音频输入/输出设备
2. 用户在会议软件中选择虚拟音频设备
3. 设置自己的语言和对方的语言
4. 用户说话时，声音被翻译成对方语言
5. 对方说话时，声音被翻译成用户语言

### 语言对配置
- 支持多种语言间的互译
- 会议中可动态切换语言对
- 例如：中文↔英语，英语↔法语，等等

## 文件结构

```
real_trans/
├── Cargo.toml          # 项目配置文件
├── Cargo.lock          # 依赖锁定文件
├── README.md           # 项目说明文档
├── SUMMARY.md          # 项目总结文档
├── build.sh            # 构建脚本
├── download_models.sh  # 下载模型脚本
├── src/                # 源代码目录
│   ├── main.rs         # 主入口点
│   ├── lib.rs          # 库入口点
│   ├── audio_types.rs  # 音频类型定义
│   ├── bidirectional_translator.rs  # 双向翻译器
│   ├── virtual_audio_manager.rs     # 虚拟音频管理器
│   ├── core/           # 核心组件
│   │   └── ring_buffer.rs         # 环形缓冲区
│   ├── engine/         # 引擎组件
│   │   ├── asr.rs      # ASR引擎
│   │   ├── mt.rs       # MT引擎
│   │   ├── vad.rs      # VAD引擎
│   │   ├── tts.rs      # TTS引擎（预留）
│   │   ├── model_loader.rs # 模型加载器
│   │   └── translation_pipeline.rs # 翻译流水线
│   ├── io/             # I/O组件
│   │   ├── audio_device.rs  # 音频设备管理
│   │   ├── audio_capture.rs # 音频捕获
│   │   └── mod.rs           # I/O模块声明
│   └── tests/          # 测试模块
│       ├── audio_simulation.rs         # 音频模拟测试
│       └── mod.rs                      # 测试模块声明
├── examples/           # 演示程序
│   ├── simple_demo.rs               # 简单演示
│   ├── integration_demo.rs          # 集成演示
│   ├── full_demo.rs                 # 完整演示
│   └── conference_translator_demo.rs # 会议翻译演示
├── bins/               # 测试程序
│   ├── simple_test.rs               # 简单测试
│   ├── integration_test.rs          # 集成测试
│   └── test_runner.rs               # 测试运行器
├── models/             # 模型目录
│   ├── whisper-tiny.bin    # Whisper ASR模型
│   └── qwen2.5-0.5b.bin  # Qwen MT模型
└── target/             # 编译输出目录（由Cargo管理）
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
- [x] 双向翻译器
- [x] 虚拟音频设备管理器
- [x] 会议翻译器演示
- [x] 音频模拟测试框架
- [x] 完整演示程序
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

运行不同级别的演示以查看功能：

```bash
# 运行简单演示
cargo run --bin simple_demo

# 运行基础集成演示
cargo run --bin integration_demo

# 运行完整演示
cargo run --bin full_demo

# 运行会议翻译器演示
cargo run --bin conference_translator_demo
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
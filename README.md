# RealTrans - 实时语音翻译系统

RealTrans是一个实时双向语音翻译系统，旨在提供流畅的跨语言交流体验。

## 项目概述

RealTrans是一个全双工实时语音翻译系统，支持双向语音翻译。它使用虚拟音频设备技术，能够同时处理发送端（用户语音翻译）和接收端（对方语音翻译）的音频流。

### 核心特性

1. **全双工双向翻译**：
   - 用户语音翻译（物理麦克风 -> 虚拟麦克风）
   - 对方语音翻译（系统环回 -> 物理耳机）

2. **虚拟音频设备管理**：
   - 自动配置虚拟音频设备
   - 支持多种音频格式和采样率

3. **实时翻译引擎**：
   - ASR（自动语音识别）
   - MT（机器翻译）
   - TTS（文本转语音）

4. **模块化设计**：
   - 音频输入/输出模块
   - 翻译引擎模块
   - 设备管理模块

## 架构设计

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   物理麦克风    │────│  虚拟音频管理器  │────│   应用程序      │
│ (用户输入)      │    │ (VirtualAudio    │    │ (语音翻译应用)  │
└─────────────────┘    │ Manager)        │    └─────────────────┘
                       └──────────────────┘
                              │
                       ┌──────────────────┐
                       │  翻译管道        │
                       │ (Translation     │
                       │ Pipeline)        │
                       └──────────────────┘
                              │
                       ┌──────────────────┐
                       │  虚拟音频管理器  │
                       │ (VirtualAudio    │
                       │ Manager)         │
                       └──────────────────┘
                              │
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   物理耳机      │◄───│  系统环回捕获    │◄───│   应用程序      │
│ (用户输出)      │    │ (System Loopback │    │ (语音翻译应用)  │
└─────────────────┘    │ Capture)        │    └─────────────────┘
                       └──────────────────┘
```

### 关键组件

1. **AudioSwitchboard**：核心音频路由组件，管理双向音频流
2. **VirtualAudioManager**：虚拟音频设备管理器
3. **BidirectionalTranslator**：双向翻译器
4. **AudioDevice**：音频设备抽象层
5. **TranslationPipeline**：翻译管道（ASR -> MT -> TTS）

## 使用说明

### 环境要求

- Rust 1.70+
- Cargo
- 相关模型文件（将在初次运行时下载）

### 构建

```bash
# 构建项目
cargo build

# 构建所有示例
cargo build --examples
```

### 运行示例

```bash
# 运行简单演示
cargo run --example simple_demo

# 运行完整演示
cargo run --example full_demo

# 运行模块测试
cargo run --example module_tests/input_test
cargo run --example module_tests/output_test
cargo run --example module_tests/input_with_translation_test
cargo run --example module_tests/full_integration_test

# 列出音频设备
cargo run --example list_audio_devices
```

## 模块介绍

### 1. IO 模块 (`src/io/`)
- `audio_device.rs`: 音频设备抽象
- `virtual_audio_device.rs`: 虚拟音频设备实现
- `audio_capture.rs`: 音频捕获组件

### 2. 引擎模块 (`src/engine/`)
- `asr.rs`: 自动语音识别
- `mt.rs`: 机器翻译
- `tts.rs`: 文本转语音
- `vad.rs`: 语音活动检测

### 3. 主要组件
- `bidirectional_translator.rs`: 双向翻译器
- `virtual_audio_manager.rs`: 虚拟音频管理器
- `audio_switchboard.rs`: 音频交换板
- `audio_types.rs`: 音频类型定义

## 配置选项

### 音频参数
- 采样率: 16000 Hz
- 声道数: 1 (单声道)
- 位深度: 16 bit
- 帧大小: 20 ms

### 语言设置
- 支持的语言对可通过配置进行修改
- 默认配置为中文-英文双向翻译

## 故障排除

参见 `TROUBLESHOOTING.md` 文件获取常见问题解决方案。

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request 来改进项目。
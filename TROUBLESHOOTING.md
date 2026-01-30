# 故障排除指南

## 常见问题

### 1. 编译错误

**问题**: `error[E0277]: cannot multiply i16 by f32`
**解决方案**: 确保在处理AudioSample时使用正确的类型。AudioSample是i16类型，不需要转换为f32再乘以i16::MAX。

**问题**: `unused import` 警告
**解决方案**: 这些是警告而非错误，不影响编译。可以通过运行 `cargo fix` 来自动修复：
```bash
cargo fix
```

### 2. 运行时问题

**问题**: 程序启动后立即退出
**解决方案**: 检查示例程序是否正确设置了音频输入回调，并且是否有持续的音频流输入。

**问题**: 虚拟音频设备无法创建
**解决方案**: 
- 检查系统是否支持虚拟音频设备
- 确保有足够的权限访问音频设备
- 检查是否有其他程序占用了音频设备

### 3. 音频相关问题

**问题**: 无法检测到音频设备
**解决方案**: 
- 确保系统中有可用的音频输入/输出设备
- 检查音频驱动程序是否正常工作
- 运行 `cargo run --example list_audio_devices` 查看可用设备

**问题**: 音频延迟过高
**解决方案**: 
- 调整 `FRAME_SIZE_MS` 参数以减少延迟
- 检查系统负载是否过高
- 确保音频缓冲区大小合适

### 4. 翻译相关问题

**问题**: 模型文件缺失
**解决方案**: 
- 确保模型文件位于 `./models/` 目录下
- 下载所需的 ASR、MT 和 TTS 模型文件
- 检查模型文件名是否与代码中指定的一致

**问题**: 翻译质量差
**解决方案**: 
- 尝试不同的模型文件
- 检查输入音频质量
- 调整翻译管道参数

## 调试技巧

### 1. 启用详细日志
在运行程序时启用详细日志输出：
```bash
RUST_LOG=debug cargo run --example simple_demo
```

### 2. 检查音频流
使用音频测试示例来验证音频流：
```bash
cargo run --example module_tests/input_test
cargo run --example module_tests/output_test
```

### 3. 验证翻译功能
单独测试翻译管道：
```bash
cargo run --example module_tests/input_with_translation_test
```

## 平台特定问题

### Linux
- 确保安装了 ALSA 开发库：`sudo apt-get install libasound2-dev`
- 检查 PulseAudio 是否正在运行

### Windows
- 确保安装了 Windows SDK
- 检查 WASAPI 或 DirectSound 驱动程序状态

### macOS
- 确保授予音频访问权限
- 检查 Core Audio 框架是否可用

## 性能优化

### 1. 内存使用
- 调整缓冲区大小以平衡内存使用和性能
- 监控音频缓冲区避免溢出

### 2. CPU 使用率
- 优化翻译管道的并发处理
- 使用适当的采样率和位深度

### 3. 延迟优化
- 减少音频帧大小
- 优化模型推理速度

## 模型文件要求

确保以下模型文件存在于 `./models/` 目录：
- `whisper-tiny.bin` - ASR 模型
- `qwen2.5-0.5b.bin` - MT 模型
- 其他必要的 TTS 模型文件

## 环境变量

- `RUST_LOG` - 控制日志级别 (trace, debug, info, warn, error)
- `AUDIO_SAMPLE_RATE` - 覆盖默认采样率 (如果支持)
- `TRANSLATION_LANGUAGES` - 指定翻译语言对

## 联系支持

如果遇到无法解决的问题，请：
1. 检查 GitHub Issues 是否已有类似问题
2. 提交新的 Issue，包含详细的错误信息和环境描述
3. 提供重现问题的最小示例代码
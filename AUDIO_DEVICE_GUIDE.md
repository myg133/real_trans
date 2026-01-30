# 音频设备选择指南

## 如何在您的电脑上选择正确的音频设备

当运行实时同声传译程序时，您需要选择合适的输入和输出设备：

### 1. 输入设备（麦克风）
- **物理麦克风**：通常是您电脑自带的麦克风或外接麦克风
  - Windows: "麦克风阵列"、"立体声混音" 或设备品牌名称
  - Mac: "内置麦克风" 或外接设备名称
  - Linux: "Built-in Audio Analog Stereo" 或具体设备名

- **虚拟音频设备**：用于截取其他应用程序的音频
  - Windows: VB-Cable Virtual Audio Device, VoiceMeeter Input
  - Mac: BlackHole, Loopback, Soundflower
  - Linux: JACK Virtual Devices

### 2. 输出设备（耳机/扬声器）
- **物理输出**：您的耳机或扬声器
  - Windows: "扬声器"、"耳机" 或 HDMI 输出
  - Mac: "内置扬声器"、"耳机" 或蓝牙设备
  - Linux: "Built-in Audio" 或具体设备名

- **虚拟音频设备**：用于将音频重定向到其他应用程序
  - Windows: VB-Cable Virtual Audio Device, VoiceMeeter Output
  - Mac: BlackHole, Loopback
  - Linux: JACK Virtual Devices

### 3. 推荐的设备配置

#### 在线会议场景：
- **输入设备**：选择您的物理麦克风（用于说话）
- **输出设备**：选择虚拟音频设备（如 VB-Cable），然后在会议软件中选择同一虚拟设备作为输入源

#### 录音场景：
- **输入设备**：选择物理麦克风
- **输出设备**：选择物理耳机（用于监听）

### 4. 设备测试步骤

1. **确定设备名称**：
   - Windows: 打开"声音设置" -> "输入" 和 "输出" 选项卡查看设备列表
   - Mac: 打开"系统偏好设置" -> "声音" 查看输入输出选项卡
   - Linux: 使用 `pactl list sinks/sources short` 命令

2. **运行设备列表程序**：
   ```bash
   cargo run --example list_audio_devices
   ```

3. **在程序中指定设备**：
   - 根据程序提示输入您的设备名称
   - 例如："麦克风 (2- USB Audio Device)" 或 "CABLE Input (VB-Audio Virtual Cable)"

### 5. 常见虚拟音频设备设置

#### Windows (推荐 VB-Audio Virtual Cable)
1. 下载并安装 [VB-Audio Virtual Cable](https://vb-audio.com/Cable/)
2. 在会议软件中，将音频输入设置为 "CABLE Input"
3. 在我们的程序中，将输出设备设置为 "CABLE Output"

#### Mac (推荐 BlackHole)
1. 下载并安装 [BlackHole](https://existential.audio/blackhole/)
2. 创建一个多输出设备，包含内置输出和 BlackHole
3. 在会议软件中选择 BlackHole 作为音频输入

### 6. 故障排除

- 如果程序无法找到您的设备，请确保：
  1. 设备驱动程序已正确安装
  2. 设备没有被其他程序独占使用
  3. 设备权限已正确授予程序

- 如果出现无声问题：
  1. 检查系统音量设置
  2. 确认选择了正确的输入/输出设备
  3. 验证设备是否正常工作（可通过录音软件测试）

### 7. 模块化测试顺序

按照以下顺序测试您的设备：

1. **输入测试**：运行 `cargo run --example input_test`
   - 选择您的物理麦克风作为输入设备
   - 说话并确认程序收到音频输入

2. **输入+翻译测试**：运行 `cargo run --example input_with_translation_test`
   - 验证语音识别和翻译功能

3. **输出测试**：运行 `cargo run --example output_test`
   - 选择您的耳机或扬声器作为输出设备
   - 验证音频播放功能

4. **完整测试**：运行 `cargo run --example full_integration_test`
   - 测试完整的双向翻译流程

---
记住：设备名称必须完全匹配系统中的实际名称，包括括号内的标识符。
# 故障排除指南

## 常见问题及解决方案

### 1. 依赖版本冲突问题

**问题描述：**
```
error: failed to select a version for the requirement `cc = "^1"` (locked to 1.2.55)
candidate versions found which didn't match: 1.2.54, 1.2.53, 1.2.52, ...
location searched: `ustc` index (which is replacing registry `crates-io`)
```

**原因：**
此问题是由于使用了USTC（中国科学技术大学）的Rust镜像源，该镜像源与官方源存在版本同步延迟，导致依赖版本不一致。

**解决方案：**

1. **检查是否有镜像源配置：**
   ```bash
   # 检查项目中是否有 .cargo/config.toml 文件
   ls .cargo/config.toml
   
   # 检查用户级别的配置
   ls ~/.cargo/config
   ```

2. **删除或修改镜像源配置：**
   如果发现 `.cargo/config.toml` 文件，请删除或编辑它：
   ```bash
   # 删除项目级配置
   rm -rf .cargo
   ```

3. **清理并重建：**
   ```bash
   cargo clean
   rm -f Cargo.lock
   cargo build
   ```

4. **如果仍存在问题，尝试使用环境变量强制使用官方源：**
   ```bash
   cargo build --offline  # 如果依赖已下载
   # 或者
   CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build
   ```

### 2. 音频设备相关问题

**问题描述：**
在某些环境下（如云开发环境、容器）无法检测到音频设备。

**解决方案：**
- 此程序设计用于有真实音频硬件的计算机
- 在服务器或云环境中，音频设备检测功能会显示"未找到任何设备"，这是正常现象
- 请在具有实际音频设备的机器上运行程序

### 3. 编译警告

本项目存在一些无害的编译警告（未使用的导入、变量等），这些不影响程序功能。如需消除警告，可运行：
```bash
cargo fix --lib
```

### 4. 运行时权限问题

在某些系统上，访问音频设备可能需要特殊权限：
- **Linux**: 可能需要将用户添加到 `audio` 组
- **Windows**: 确保应用程序有麦克风和音频访问权限
- **MacOS**: 在系统偏好设置中授予终端或IDE麦克风访问权限

### 5. 虚拟音频设备配置

如需在会议软件中使用此翻译系统，请参考 [AUDIO_DEVICE_GUIDE.md](AUDIO_DEVICE_GUIDE.md) 中的详细配置指南。

---

**注意：** 如果遇到其他问题，请查看相关错误信息并参考 Rust 和 cpal 音频库的官方文档。
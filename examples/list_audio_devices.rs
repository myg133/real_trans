//! 音频设备列表示例
//! 展示如何列出系统上的所有音频设备

use real_trans::io::physical_device::PhysicalAudioDevice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 搜索可用的音频输入设备...");
    
    match PhysicalAudioDevice::list_input_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("❌ 未找到任何输入设备");
            } else {
                println!("✅ 找到 {} 个输入设备:", devices.len());
                for (i, device) in devices.iter().enumerate() {
                    let default_mark = if device.is_default { " (默认)" } else { "" };
                    println!("  {}. {} [ID: {}]{}", 
                        i + 1, 
                        device.name, 
                        device.id, 
                        default_mark
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ 获取输入设备失败: {}", e);
        }
    }

    println!("\n🔍 搜索可用的音频输出设备...");
    
    match PhysicalAudioDevice::list_output_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("❌ 未找到任何输出设备");
            } else {
                println!("✅ 找到 {} 个输出设备:", devices.len());
                for (i, device) in devices.iter().enumerate() {
                    let default_mark = if device.is_default { " (默认)" } else { "" };
                    println!("  {}. {} [ID: {}]{}", 
                        i + 1, 
                        device.name, 
                        device.id, 
                        default_mark
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ 获取输出设备失败: {}", e);
        }
    }

    println!("\n💡 提示:");
    println!("  - 通常您的物理麦克风会显示为 '麦克风'、'Microphone' 或类似名称");
    println!("  - 通常您的物理耳机/扬声器会显示为 '扬声器'、'Headphones' 或类似名称");
    println!("  - 虚拟音频设备（如VB-Cable）也会在此列表中显示");
    println!("  - 请记下您想要使用的设备名称，在程序中选择它们");

    Ok(())
}
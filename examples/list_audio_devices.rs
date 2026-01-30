//! 音频设备列表示例
//! 展示如何列出系统上的所有音频设备

use cpal::traits::{DeviceTrait, HostTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    
    println!("🔍 搜索可用的音频输入设备...");
    
    match host.input_devices() {
        Ok(mut devices) => {
            let input_devices: Vec<_> = devices.collect();
            if input_devices.is_empty() {
                println!("❌ 未找到任何输入设备");
            } else {
                println!("✅ 找到 {} 个输入设备:", input_devices.len());
                for (i, device) in input_devices.iter().enumerate() {
                    let device_name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
                    let default_mark = if let Some(default_device) = host.default_input_device() {
                        if default_device.name().unwrap_or_default() == device.name().unwrap_or_default() {
                            " (默认)"
                        } else {
                            ""
                        }
                    } else {
                        ""
                    };
                    println!("  {}. {}{}", 
                        i + 1, 
                        device_name,
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
    
    match host.output_devices() {
        Ok(mut devices) => {
            let output_devices: Vec<_> = devices.collect();
            if output_devices.is_empty() {
                println!("❌ 未找到任何输出设备");
            } else {
                println!("✅ 找到 {} 个输出设备:", output_devices.len());
                for (i, device) in output_devices.iter().enumerate() {
                    let device_name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
                    let default_mark = if let Some(default_device) = host.default_output_device() {
                        if default_device.name().unwrap_or_default() == device.name().unwrap_or_default() {
                            " (默认)"
                        } else {
                            ""
                        }
                    } else {
                        ""
                    };
                    println!("  {}. {}{}", 
                        i + 1, 
                        device_name,
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
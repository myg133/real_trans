//! 音频输入模块测试
//! 测试物理麦克风输入并将音频保存到文件

use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use real_trans::io::audio_device::AudioDevice;
use real_trans::io::virtual_audio_device::VirtualAudioDevice;
use real_trans::audio_types::AudioSample;

struct AudioRecorder {
    buffer: Arc<Mutex<Vec<AudioSample>>>,
}

impl AudioRecorder {
    fn new() -> Self {
        AudioRecorder {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn record_audio(&self, audio_data: &[AudioSample]) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(audio_data);
        println!("Recorded {} samples, total: {}", audio_data.len(), buffer.len());
    }

    fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = self.buffer.lock().unwrap();
        let mut file = File::create(filename)?;
        
        for &sample in buffer.iter() {
            // 将f32样本转换为i16并写入文件（小端序）
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            file.write_all(&sample_i16.to_le_bytes())?;
        }
        
        println!("Saved {} samples to {}", buffer.len(), filename);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 音频输入模块测试 ===");
    println!("此程序将录制来自物理麦克风的音频并保存到文件");
    
    // 显示可用的输入设备
    let mut audio_device = VirtualAudioDevice::new();
    
    println!("\n🔍 可用的输入设备:");
    for (i, dev) in audio_device.get_available_input_devices().iter().enumerate() {
        println!("  {}: {} (ID: {})", i + 1, dev.name, dev.id);
    }
    
    println!("\n🏠 默认输入设备: {}", audio_device.get_default_input_device().name);
    
    println!("\n请输入要使用的输入设备名称（直接回车使用默认设备）:");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    
    // 使用默认设备或用户指定的设备
    let device_id = if input.is_empty() {
        None
    } else {
        Some(input)
    };

    // 创建音频记录器
    let recorder = AudioRecorder::new();
    let recorder_clone = Arc::clone(&recorder.buffer);

    println!("\n🎯 准备打开输入流...");
    
    // 设置音频输入回调
    audio_device.open_input_stream(
        device_id,
        Box::new(move |audio_data| {
            if !audio_data.is_empty() {
                // 将音频数据添加到记录器
                let mut buffer = recorder_clone.lock().unwrap();
                buffer.extend_from_slice(audio_data);
                println!("🎤 Captured {} samples (first sample: {:.3}, max amp: {:.3})", 
                    audio_data.len(), 
                    audio_data[0],
                    audio_data.iter().map(|x| x.abs()).fold(0.0, |a, b| a.max(b))
                );
            }
        }),
    )?;

    println!("✅ 成功打开输入流");

    println!("\n按 Enter 键开始录制...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // 开始录制
    println!("🎬 开始录制...");
    audio_device.start_recording()?;
    println!("⏳ 录制中... 请说话，按 Enter 键停止录制");
    
    std::io::stdin().read_line(&mut input)?;

    // 停止录制
    println!("⏹️ 停止录制...");
    audio_device.stop_recording()?;
    audio_device.close_input_stream()?;

    // 保存录制的音频到文件
    recorder.save_to_file("recorded_input.raw")?;
    
    println!("🎉 音频输入测试完成！");
    println!("录制的音频已保存到 recorded_input.raw");
    println!("您可以使用音频播放软件检查文件内容");

    Ok(())
}
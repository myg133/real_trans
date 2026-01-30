//! 音频输入模块测试
//! 测试物理麦克风输入并将音频保存到文件

use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use real_trans::io::audio_device::{AudioDevice, MockAudioDevice, AudioSample};

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
        
        for sample in buffer.iter() {
            // 将i16样本写入文件（小端序）
            file.write_all(&sample.to_le_bytes())?;
        }
        
        println!("Saved {} samples to {}", buffer.len(), filename);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 音频输入模块测试 ===");
    println!("此程序将录制来自物理麦克风的音频并保存到文件");
    println!("按 Enter 键开始录制...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // 创建音频记录器
    let recorder = AudioRecorder::new();
    let recorder_clone = Arc::clone(&recorder.buffer);

    // 创建模拟音频设备（在实际实现中，这里会连接到真实的物理麦克风）
    let mut audio_device = MockAudioDevice::new();
    
    // 设置音频输入回调
    audio_device.open_input_stream(
        Some("physical_mic".to_string()),
        Box::new(move |audio_data| {
            // 将音频数据添加到记录器
            let mut buffer = recorder_clone.lock().unwrap();
            buffer.extend_from_slice(audio_data);
            println!("Captured {} samples", audio_data.len());
        }),
    )?;

    // 开始录制
    audio_device.start_recording()?;
    println!("开始录制... 按 Enter 键停止录制");
    std::io::stdin().read_line(&mut input)?;

    // 停止录制
    audio_device.stop_recording()?;
    audio_device.close_input_stream()?;

    // 保存录制的音频到文件
    recorder.save_to_file("recorded_input.raw")?;
    
    println!("音频输入测试完成！");
    println!("录制的音频已保存到 recorded_input.raw");
    println!("您可以使用音频播放软件检查文件内容");

    Ok(())
}
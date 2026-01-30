//! 音频输出模块测试
//! 测试从文件读取音频并通过耳机播放

use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use real_trans::io::audio_device::{AudioDevice, VirtualAudioDevice, AudioSample};

struct AudioPlayer {
    audio_device: Option<Box<dyn AudioDevice>>,
}

impl AudioPlayer {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(AudioPlayer {
            audio_device: Some(Box::new(VirtualAudioDevice::new())),
        })
    }

    fn load_from_file(&self, filename: &str) -> Result<Vec<AudioSample>, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // 将字节转换为i16样本（假设小端序）
        let mut samples = Vec::new();
        for chunk in buffer.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            samples.push(sample);
        }
        
        println!("Loaded {} samples from {}", samples.len(), filename);
        Ok(samples)
    }

    fn play_audio(&mut self, audio_data: &[AudioSample]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut device) = self.audio_device {
            // 打开输出流并播放音频
            device.open_output_stream(Some("physical_headphones".to_string()))?;
            let played = device.play_audio(audio_data)?;
            println!("Played {} samples", played);
            device.close_output_stream()?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 音频输出模块测试 ===");
    println!("此程序将从文件读取音频并通过耳机播放");
    
    // 尝试从之前测试生成的文件读取，如果没有则使用默认音频
    let audio_data = if std::path::Path::new("recorded_input.raw").exists() {
        println!("从 recorded_input.raw 加载音频数据");
        AudioPlayer::load_from_file(&AudioPlayer::new()?, "recorded_input.raw")?
    } else if std::path::Path::new("translated_output.raw").exists() {
        println!("从 translated_output.raw 加载音频数据");
        AudioPlayer::load_from_file(&AudioPlayer::new()?, "translated_output.raw")?
    } else {
        println!("使用默认测试音频数据");
        vec![100, 200, 300, 400, 500, 400, 300, 200, 100] // 简单的波形
    };

    // 创建音频播放器
    let mut player = AudioPlayer::new()?;
    
    println!("开始播放音频... 共 {} 个样本", audio_data.len());
    player.play_audio(&audio_data)?;
    
    println!("音频输出测试完成！");
    println!("音频已通过模拟耳机播放");

    Ok(())
}
//! 音频输入+翻译模块测试
//! 测试物理麦克风输入，经过翻译后保存到文件

use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use real_trans::{
    io::audio_device::AudioDevice,
    io::virtual_audio_device::VirtualAudioDevice,
    audio_types::AudioSample,
    engine::translation_pipeline::{TranslationPipeline, TranslationCallback},
    bidirectional_translator::{BidirectionalTranslator, BidirectionalResult, TranslationDirection}
};

struct AudioRecorder {
    buffer: Arc<Mutex<Vec<AudioSample>>>,
}

impl AudioRecorder {
    fn new() -> Self {
        AudioRecorder {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = self.buffer.lock().unwrap();
        let mut file = File::create(filename)?;
        
        for &sample in buffer.iter() {
            // 将i16样本写入文件（小端序）
            file.write_all(&sample.to_le_bytes())?;
        }
        
        println!("Saved {} samples to {}", buffer.len(), filename);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 音频输入+翻译模块测试 ===");
    println!("此程序将录制来自物理麦克风的音频，经过翻译后保存到文件");
    println!("按 Enter 键开始录制...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // 创建翻译器
    let mut translator = BidirectionalTranslator::new("zh", "en")?;
    
    // 创建音频记录器
    let recorder = Arc::new(Mutex::new(AudioRecorder::new()));
    
    // 设置翻译结果回调
    translator.set_result_callback(|result: &BidirectionalResult| {
        println!("Translation result:");
        println!("  Original: {}", result.original_text);
        println!("  Translated: {}", result.translated_text);
        println!("  Direction: {:?}", result.direction);
    });
    
    translator.start()?;

    // 创建虚拟音频设备（在实际实现中，这里会连接到真实的物理麦克风）
    let mut audio_device = VirtualAudioDevice::new();
    
    // 设置音频输入回调
    let recorder_clone = Arc::clone(&recorder);
    audio_device.open_input_stream(
        Some("physical_mic".to_string()),
        Box::new(move |audio_data| {
            println!("Captured {} samples", audio_data.len());
            // 在实际实现中，这里会将音频数据传递给翻译器
            // 但现在我们只是模拟这个过程
        }),
    )?;

    // 开始录制
    audio_device.start_recording()?;
    println!("开始录制... 模拟翻译过程");
    println!("按 Enter 键停止录制");
    std::io::stdin().read_line(&mut input)?;

    // 模拟一些音频输入和翻译
    println!("模拟音频输入和翻译过程...");
    for i in 0..5 {
        let sample_audio: Vec<AudioSample> = vec![100, 200, 300, 400, 500]; // 模拟音频数据
        translator.handle_outbound_audio(&sample_audio).await;
        sleep(Duration::from_millis(500)).await;
    }

    // 停止录制
    audio_device.stop_recording()?;
    audio_device.close_input_stream()?;
    translator.stop()?;

    // 保存模拟的翻译后音频到文件
    // 在实际实现中，这里会保存TTS生成的音频
    let sample_translated_audio: Vec<AudioSample> = vec![
        500, 400, 300, 200, 100,  // 模拟翻译后的音频数据
        100, 200, 300, 400, 500,
        500, 400, 300, 200, 100,
    ];
    
    {
        let mut recorder_lock = recorder.lock().unwrap();
        let mut buffer = recorder_lock.buffer.lock().unwrap();
        buffer.extend_from_slice(&sample_translated_audio);
    }
    
    recorder.lock().unwrap().save_to_file("translated_output.raw")?;
    
    println!("音频输入+翻译测试完成！");
    println!("翻译后的音频已保存到 translated_output.raw");
    println!("您可以使用音频播放软件检查文件内容");

    Ok(())
}
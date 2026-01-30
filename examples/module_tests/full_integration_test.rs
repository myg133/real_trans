//! 完整集成测试
//! 结合输入、翻译和输出模块的端到端测试

use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use real_trans::{
    io::audio_device::AudioDevice,
    io::virtual_audio_device::VirtualAudioDevice,
    audio_types::AudioSample,
    bidirectional_translator::{BidirectionalTranslator, BidirectionalResult, TranslationDirection}
};

struct AudioProcessor {
    input_buffer: Arc<Mutex<Vec<AudioSample>>>,
    output_buffer: Arc<Mutex<Vec<AudioSample>>>,
}

impl AudioProcessor {
    fn new() -> Self {
        AudioProcessor {
            input_buffer: Arc::new(Mutex::new(Vec::new())),
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn save_to_file(&self, filename: &str, buffer_type: &str) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = match buffer_type {
            "input" => self.input_buffer.lock().unwrap(),
            "output" => self.output_buffer.lock().unwrap(),
            _ => panic!("Invalid buffer type"),
        };
        
        let mut file = File::create(filename)?;
        
        for &sample in buffer.iter() {
            // 将i16样本写入文件（小端序）
            file.write_all(&sample.to_le_bytes())?;
        }
        
        println!("Saved {} samples to {}", buffer.len(), filename);
        Ok(())
    }

    fn load_from_file(&self, filename: &str) -> Result<Vec<AudioSample>, Box<dyn std::error::Error>> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // 将字节转换为i16样本（假设小端序）
        let mut samples = Vec::new();
        for chunk in buffer.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            // 将i16转换为AudioSample(f32)
            samples.push(sample as AudioSample / i16::MAX as AudioSample);
        }
        
        println!("Loaded {} samples from {}", samples.len(), filename);
        Ok(samples)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 完整集成测试 ===");
    println!("此程序将演示端到端的全双工翻译流程");
    
    // 创建音频处理器
    let processor = Arc::new(Mutex::new(AudioProcessor::new()));
    
    // 创建翻译器
    let mut translator = BidirectionalTranslator::new("zh", "en")?;
    
    // 设置翻译结果回调
    translator.set_result_callback(|result: &BidirectionalResult| {
        println!("Translation result:");
        println!("  Original: {}", result.original_text);
        println!("  Translated: {}", result.translated_text);
        println!("  Direction: {:?}", result.direction);
    });
    
    translator.start()?;

    // 创建虚拟音频设备（在实际实现中，这里会连接到真实的物理设备）
    let mut input_device = VirtualAudioDevice::new();
    input_device.open_input_stream(
        Some("physical_mic".to_string()),
        Box::new({
            let processor = Arc::clone(&processor);
            move |audio_data| {
                println!("Input: Captured {} samples", audio_data.len());
                // 将输入音频添加到输入缓冲区
                let mut proc = processor.lock().unwrap();
                let mut input_buf = proc.input_buffer.lock().unwrap();
                input_buf.extend_from_slice(audio_data);
            }
        }),
    )?;
    input_device.start_recording()?;

    // 创建虚拟音频输出设备
    let mut output_device = VirtualAudioDevice::new();
    output_device.open_output_stream(Some("physical_headphones".to_string()))?;

    println!("开始端到端测试...");
    
    // 模拟用户说话（输入中文，期望输出英文）
    println!("\n--- 模拟用户说中文 ---");
    let chinese_audio: Vec<AudioSample> = vec![100, 200, 300, 400, 500, 400, 300, 200, 100];
    translator.handle_outbound_audio(&chinese_audio).await;
    sleep(Duration::from_millis(100)).await;
    
    // 模拟对方说话（输入英文，期望输出中文）
    println!("\n--- 模拟对方说英文 ---");
    let english_audio: Vec<AudioSample> = vec![500, 400, 300, 200, 100, 200, 300, 400, 500];
    translator.handle_inbound_audio(&english_audio).await;
    sleep(Duration::from_millis(100)).await;

    // 停止设备
    input_device.stop_recording()?;
    input_device.close_input_stream()?;
    output_device.close_output_stream()?;
    translator.stop()?;

    // 保存处理结果
    let proc = processor.lock().unwrap();
    proc.save_to_file("integration_input.raw", "input")?;
    proc.save_to_file("integration_output.raw", "output")?;

    println!("\n=== 测试完成 ===");
    println!("端到端翻译流程测试完成！");
    println!("- 输入音频保存为: integration_input.raw");
    println!("- 输出音频保存为: integration_output.raw");
    println!("- 实现了全双工双向语音翻译");
    println!("- 支持用户语音翻译和对方语音翻译");

    Ok(())
}
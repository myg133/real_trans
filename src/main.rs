//! 实时同声翻译应用（Real-time Speech Translation, RST）
//! 主程序入口

use real_trans::engine::translation_pipeline::{TranslationPipeline, TranslationResult};
use real_trans::audio_types::{AudioSample, SAMPLES_PER_FRAME};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("实时同声翻译应用 (Real-time Speech Translation, RST)");
    println!("正在初始化...");

    // 初始化翻译流水线
    // 注意：在实际部署时，需要提供正确的模型路径
    let asr_model_path = "./models/whisper-tiny.bin".to_string();
    let mt_model_path = "./models/qwen2.5-0.5b.bin".to_string();
    
    let mut pipeline = TranslationPipeline::new(asr_model_path, mt_model_path);
    
    // 设置翻译回调函数
    pipeline.set_translation_callback(Box::new(|result: &TranslationResult| {
        println!("\n=== 翻译结果 ===");
        println!("原文: {}", result.original_text);
        println!("译文: {}", result.translated_text);
        println!("ASR置信度: {:.2}", result.asr_confidence);
        println!("MT置信度: {:.2}", result.mt_confidence);
        println!("类型: {}", if result.is_final { "最终结果" } else { "中间结果" });
        println!("================\n");
    }));

    // 初始化流水线
    if let Err(e) = pipeline.initialize() {
        eprintln!("初始化失败: {}", e);
        return;
    }

    println!("初始化完成，正在启动翻译流水线...");

    // 启动流水线
    if let Err(e) = pipeline.start() {
        eprintln!("启动流水线失败: {}", e);
        return;
    }

    println!("流水线已启动，正在模拟音频输入...");

    // 模拟音频数据（实际应用中这里来自麦克风）
    let sample_count = SAMPLES_PER_FRAME * 5; // 5个音频帧的数据
    let mut mock_audio = vec![0i16; sample_count];
    
    // 生成模拟音频数据（简单的递增序列）
    for i in 0..sample_count {
        mock_audio[i] = (i % 1000) as i16; // 简单的模拟音频数据
    }

    // 模拟处理音频数据
    for i in 0..5 {
        println!("处理音频块 {}/5...", i + 1);
        
        // 获取当前音频帧
        let start_idx = i * SAMPLES_PER_FRAME;
        let end_idx = start_idx + SAMPLES_PER_FRAME;
        let audio_frame = &mock_audio[start_idx..end_idx];
        
        // 处理音频帧
        pipeline.process_frame(audio_frame);
        
        sleep(Duration::from_millis(200)).await;
    }

    // 模拟一些静音时间，以便触发最终结果
    println!("等待最终结果...");
    sleep(Duration::from_secs(1)).await;

    // 停止流水线
    if let Err(e) = pipeline.stop() {
        eprintln!("停止流水线失败: {}", e);
    }

    println!("程序已退出。");
}
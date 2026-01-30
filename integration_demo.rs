//! 集成演示 - 展示如何将所有组件组合在一起

use real_trans::{
    engine::translation_pipeline::{TranslationPipeline, TranslationResult},
    audio_types::{AudioSample, SAMPLES_PER_FRAME},
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== 实时同声翻译应用集成演示 ===\n");

    // 1. 演示模型加载
    println!("1. 加载模型组件...");
    let asr_model_path = "./models/whisper-tiny.bin".to_string();
    let mt_model_path = "./models/qwen2.5-0.5b.bin".to_string();
    
    println!("   ASR模型路径: {}", asr_model_path);
    println!("   MT模型路径: {}", mt_model_path);
    
    // 注意：在实际部署时，需要提供正确的模型路径
    // 模型可以通过download_models.sh脚本下载
    
    // 2. 创建翻译流水线
    println!("\n2. 创建翻译流水线...");
    let mut pipeline = TranslationPipeline::new(asr_model_path, mt_model_path);
    
    // 3. 设置翻译回调函数
    pipeline.set_translation_callback(|result: &TranslationResult| {
        println!("\n>>> 翻译结果 <<<");
        println!("原文: {}", result.original_text);
        println!("译文: {}", result.translated_text);
        println!("ASR置信度: {:.2}", result.asr_confidence);
        println!("MT置信度: {:.2}", result.mt_confidence);
        println!("类型: {}", if result.is_final { "最终结果" } else { "中间结果" });
        println!("时间: {:?}", result.timestamp.elapsed());
        println!("-------------------\n");
    });

    // 4. 初始化流水线
    println!("3. 初始化翻译流水线...");
    match pipeline.initialize() {
        Ok(_) => println!("   ✓ 流水线初始化成功"),
        Err(e) => {
            println!("   ✗ 流水线初始化失败: {}", e);
            return;
        }
    }

    // 5. 启动流水线
    println!("4. 启动翻译流水线...");
    match pipeline.start() {
        Ok(_) => println!("   ✓ 流水线启动成功"),
        Err(e) => {
            println!("   ✗ 流水线启动失败: {}", e);
            return;
        }
    }

    // 6. 模拟音频输入和处理
    println!("5. 模拟音频处理...");
    println!("   正在生成模拟音频数据并处理...");
    
    // 模拟音频数据（实际应用中这里来自麦克风）
    let sample_count = SAMPLES_PER_FRAME * 5; // 5个音频帧的数据
    let mut mock_audio = vec![0i16; sample_count];
    
    // 生成模拟音频数据（简单的正弦波）
    for i in 0..sample_count {
        // 使用简单的数学函数生成模拟音频
        mock_audio[i] = (i as i16 % 1000 - 500) as i16;
    }

    // 模拟处理音频数据
    for i in 0..5 {
        println!("   处理音频块 {}/5...", i + 1);
        
        // 获取当前音频帧
        let start_idx = i * SAMPLES_PER_FRAME;
        let end_idx = start_idx + SAMPLES_PER_FRAME;
        let audio_frame = &mock_audio[start_idx..end_idx];
        
        // 处理音频帧
        pipeline.process_frame(audio_frame);
        
        sleep(Duration::from_millis(200)).await;
    }

    // 模拟一些静音时间，以便触发最终结果
    println!("6. 等待最终结果...");
    sleep(Duration::from_secs(1)).await;

    // 7. 停止流水线
    println!("7. 停止翻译流水线...");
    match pipeline.stop() {
        Ok(_) => println!("   ✓ 流水线已停止"),
        Err(e) => println!("   ✗ 停止流水线时出错: {}", e),
    }

    println!("\n=== 演示完成 ===");
    println!("\n要运行完整的实时翻译应用，请:");
    println!("1. 下载模型文件: ./download_models.sh");
    println!("2. 更新模型路径为实际的模型文件路径");
    println!("3. 运行: cargo run");
    println!("\n系统特性:");
    println!("- 低延迟音频处理");
    println!("- 实时语音识别 (ASR)");
    println!("- 实时机器翻译 (MT)");
    println!("- 语音活动检测 (VAD)");
    println!("- 跨平台支持");
}
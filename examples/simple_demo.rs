//! 简化的实时翻译演示

use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use real_trans::{
    virtual_audio_manager::AppContext,
    bidirectional_translator::{BidirectionalResult, TranslationDirection},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 实时翻译系统演示 ===\n");

    // 1. 创建应用程序上下文
    println!("1. 初始化应用程序上下文...");
    let mut app_context = AppContext::new("zh", "en")?;
    app_context.initialize()?;

    // 2. 设置翻译结果处理器
    app_context.set_translation_handler(|result: &BidirectionalResult| {
        println!("翻译结果: 方向={:?}, 原文='{}', 译文='{}'", 
                 result.direction, 
                 result.original_text, 
                 result.translated_text);
    });

    // 3. 启动系统
    println!("2. 启动实时翻译系统...");
    app_context.start()?;

    // 4. 模拟一些音频输入
    println!("3. 模拟音频输入测试...");
    
    // 模拟用户说话（中文）
    println!("   模拟用户说中文...");
    let chinese_audio = vec![0i16; 100]; // 模拟音频数据
    app_context.simulate_user_speaking(&chinese_audio).await;
    
    sleep(Duration::from_millis(500)).await;

    // 模拟对方说话（英文）
    println!("   模拟对方说英文...");
    let english_audio = vec![0i16; 100]; // 模拟音频数据
    app_context.simulate_other_speaking(&english_audio);
    
    sleep(Duration::from_millis(500)).await;

    // 5. 等待处理完成
    println!("4. 等待处理完成...");
    sleep(Duration::from_secs(2)).await;

    // 6. 停止系统
    println!("5. 停止实时翻译系统...");
    app_context.stop()?;

    println!("\n=== 演示完成 ===");

    Ok(())
}
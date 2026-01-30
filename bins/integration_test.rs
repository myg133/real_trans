//! 实时翻译系统集成测试

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use real_trans::{
    virtual_audio_manager::AppContext,
    bidirectional_translator::{BidirectionalResult, TranslationDirection},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 实时翻译系统集成测试 ===\n");

    // 1. 创建应用程序上下文
    println!("1. 初始化应用程序上下文...");
    let mut app_context = AppContext::new("zh", "en")?;
    app_context.initialize()?;

    // 2. 设置翻译结果处理器
    let result_handler = |result: &BidirectionalResult| {
        println!("翻译结果: 方向={:?}, 原文='{}', 译文='{}'", 
                 result.direction, 
                 result.original_text, 
                 result.translated_text);
    };
    app_context.set_translation_handler(result_handler);

    // 3. 启动系统
    println!("2. 启动实时翻译系统...");
    app_context.start()?;

    // 4. 运行一段时间以允许系统稳定
    println!("3. 系统预热中...");
    sleep(Duration::from_secs(1)).await;

    // 5. 测试各种功能
    println!("4. 执行功能测试...");
    
    // 测试方向切换
    println!("   切换到用户说话模式...");
    app_context.switch_to_user_mode();
    
    // 测试音频处理模拟
    println!("   模拟用户音频输入...");
    let audio_data = vec![0i16; 100];
    app_context.simulate_user_speaking(&audio_data).await;
    
    sleep(Duration::from_millis(300)).await;
    
    println!("   模拟其他方音频输入...");
    app_context.simulate_other_speaking(&audio_data);
    
    sleep(Duration::from_millis(300)).await;

    // 6. 测试配置更改
    println!("5. 测试配置更改...");
    app_context.update_language_pair("en", "zh")?;
    println!("   语言对已更改为: 英语 -> 中文");

    // 7. 再次测试翻译
    println!("6. 执行第二次测试...");
    app_context.simulate_user_speaking(&audio_data).await;
    sleep(Duration::from_millis(300)).await;

    // 8. 获取统计信息
    println!("7. 获取系统统计信息...");
    let stats = app_context.get_statistics();
    println!("   总处理次数: {}", stats.total_processed);
    println!("   成功翻译次数: {}", stats.successful_translations);
    println!("   错误次数: {}", stats.error_count);
    println!("   平均延迟: {:.2}ms", stats.avg_latency_ms);

    // 9. 停止系统
    println!("8. 停止实时翻译系统...");
    app_context.stop()?;

    println!("\n=== 集成测试完成 ===");

    Ok(())
}
//! 完整的实时翻译系统演示

use std::time::Instant;
use tokio::time::{sleep, Duration};
use real_trans::{
    virtual_audio_manager::AppContext,
    bidirectional_translator::{BidirectionalResult, TranslationDirection},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║           实时翻译系统完整演示                   ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    // 1. 创建应用程序上下文
    println!("🔄 初始化应用程序上下文...");
    let mut app_context = AppContext::new("zh", "en")?;
    app_context.initialize()?;

    // 2. 设置翻译结果处理器
    app_context.set_translation_handler(|result: &BidirectionalResult| {
        let now = Instant::now();
        println!("💬 翻译结果 [{}ms]:", 
                 result.timestamp.elapsed().as_millis());
        println!("   方向: {:?}", result.direction);
        println!("   原文: '{}'", result.original_text);
        println!("   译文: '{}'", result.translated_text);
        println!("   时间: {:?}", now);
        println!();
    });

    // 3. 启动系统
    println!("🚀 启动实时翻译系统...");
    app_context.start()?;

    println!("\n📋 系统信息:");
    let lang_pair = app_context.get_current_language_pair();
    println!("   语言对: {} ↔ {}", lang_pair.source, lang_pair.target);
    println!("   状态: 正在运行");

    // 4. 演示各种功能
    println!("\n🎯 开始功能演示...");
    
    // 演示1: 用户说中文
    println!("\n📝 演示1: 用户说中文 (自动翻译成英文)");
    let chinese_audio = vec![0i16; 100]; // 模拟中文音频
    app_context.simulate_user_speaking(&chinese_audio).await;
    sleep(Duration::from_millis(500)).await;

    // 演示2: 对方说英文
    println!("\n📝 演示2: 对方说英文 (自动翻译成中文)");
    let english_audio = vec![0i16; 100]; // 模拟英文音频
    app_context.simulate_other_speaking(&english_audio);
    sleep(Duration::from_millis(500)).await;

    // 演示3: 更改语言对
    println!("\n🌍 演示3: 更改语言对为 日语 ↔ 韩语");
    app_context.update_language_pair("ja", "ko")?;
    let new_lang_pair = app_context.get_current_language_pair();
    println!("   新语言对: {} ↔ {}", new_lang_pair.source, new_lang_pair.target);

    // 演示4: 使用新语言对
    println!("\n📝 演示4: 使用新语言对进行翻译");
    app_context.simulate_user_speaking(&chinese_audio).await;
    sleep(Duration::from_millis(500)).await;

    // 演示5: 切换到用户模式
    println!("\n🔄 演示5: 切换到用户说话模式");
    app_context.switch_to_user_mode();
    app_context.simulate_user_speaking(&english_audio).await;
    sleep(Duration::from_millis(500)).await;

    // 演示6: 获取统计信息
    println!("\n📊 演示6: 系统统计信息");
    let stats = app_context.get_statistics();
    println!("   总处理数: {}", stats.total_processed);
    println!("   成功翻译: {}", stats.successful_translations);
    println!("   错误次数: {}", stats.error_count);
    println!("   平均延迟: {:.2}ms", stats.avg_latency_ms);

    // 7. 等待处理完成
    println!("\n⏳ 等待所有处理完成...");
    sleep(Duration::from_secs(2)).await;

    // 8. 停止系统
    println!("\n🛑 停止实时翻译系统...");
    app_context.stop()?;

    println!("\n🎉 演示完成!");
    println!("感谢使用实时翻译系统演示。");

    Ok(())
}
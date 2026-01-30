//! 会议翻译器演示
//! 展示如何使用虚拟音频设备实现双向实时翻译

use real_trans::{
    virtual_audio_manager::AppContext,
    AudioSample
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== 实时会议翻译器演示 ===\n");

    // 1. 创建应用程序上下文（用户说中文，对方说英文）
    println!("1. 初始化应用程序...");
    let mut app = AppContext::new("zh", "en")
        .expect("Failed to create application context");

    // 2. 初始化应用程序
    println!("2. 初始化应用程序...");
    app.initialize()
        .expect("Failed to initialize application");

    // 3. 显示当前语言设置
    let lang_pair = app.get_current_language_pair();
    println!("   用户语言: {}", lang_pair.source);
    println!("   对方语言: {}", lang_pair.target);

    // 4. 启动应用程序
    println!("\n3. 启动应用程序...");
    app.start()
        .expect("Failed to start application");

    // 5. 模拟会议场景
    println!("\n4. 开始模拟会议场景...");
    println!("   用户和对方交替发言，系统实时翻译...");

    // 模拟用户说中文
    println!("\n   用户开始说话 (中文 -> 英文翻译)...");
    let user_audio: Vec<AudioSample> = vec![100, 200, 300, 400, 500]; // 模拟音频数据
    app.simulate_user_speaking(&user_audio).await;
    sleep(Duration::from_millis(500)).await;

    // 模拟对方说英文
    println!("\n   对方开始说话 (英文 -> 中文翻译)...");
    let other_audio: Vec<AudioSample> = vec![500, 400, 300, 200, 100]; // 模拟音频数据
    app.simulate_other_speaking(&other_audio);
    sleep(Duration::from_millis(500)).await;

    // 再次模拟用户说话
    println!("\n   用户再次说话 (中文 -> 英文翻译)...");
    let user_audio2: Vec<AudioSample> = vec![150, 250, 350, 450, 550];
    app.simulate_user_speaking(&user_audio2).await;
    sleep(Duration::from_millis(500)).await;

    // 模拟对方回应
    println!("\n   对方回应 (英文 -> 中文翻译)...");
    let other_audio2: Vec<AudioSample> = vec![550, 450, 350, 250, 150];
    app.simulate_other_speaking(&other_audio2);
    sleep(Duration::from_millis(500)).await;

    // 6. 演示语言切换
    println!("\n5. 演示语言切换...");
    println!("   将语言对从 zh->en 切换到 en->fr");
    app.update_languages("en", "fr")
        .expect("Failed to update languages");
    
    let new_lang_pair = app.get_current_language_pair();
    println!("   新的用户语言: {}", new_lang_pair.source);
    println!("   新的对方语言: {}", new_lang_pair.target);

    // 模拟新的语言场景
    println!("\n   用户用英语说话 (英语 -> 法语翻译)...");
    let user_audio3: Vec<AudioSample> = vec![120, 220, 320, 420, 520];
    app.simulate_user_speaking(&user_audio3).await;
    sleep(Duration::from_millis(500)).await;

    println!("\n   对方用法语回应 (法语 -> 英语翻译)...");
    let other_audio3: Vec<AudioSample> = vec![520, 420, 320, 220, 120];
    app.simulate_other_speaking(&other_audio3);
    sleep(Duration::from_millis(500)).await;

    // 7. 停止应用程序
    println!("\n6. 停止应用程序...");
    app.stop()
        .expect("Failed to stop application");

    println!("\n=== 演示完成 ===");
    println!("\n系统功能总结:");
    println!("- 虚拟音频输入/输出设备");
    println!("- 用户语言: {}", lang_pair.source);
    println!("- 对方语言: {}", lang_pair.target);
    println!("- 双向实时翻译");
    println!("- 在线会议集成支持");
    println!("- 动态语言切换");
    println!("- 低延迟处理");
}
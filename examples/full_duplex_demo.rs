//! 全双工实时双向语音同传演示
//! 展示新的音频路由架构

use real_trans::{
    audio_switchboard::{AudioSwitchboard, AudioControl},
    AudioSample,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 全双工实时双向语音同传演示 ===\n");

    // 1. 创建音频交换机（支持中英双向实时翻译）
    println!("1. 初始化音频交换机...");
    let mut switchboard = AudioSwitchboard::new("zh", "en")?;
    
    // 2. 启动音频交换机
    println!("2. 启动全双工翻译系统...");
    switchboard.start().await?;
    
    println!("系统状态: {:?}", switchboard.get_status());
    println!("音频设备已配置:");
    println!("  - 物理麦克风 -> 虚拟麦克风（发送端流水线）");
    println!("  - 系统环回 -> 物理耳机（接收端流水线）");
    println!("  - 支持实时双向语音翻译");

    // 3. 模拟一些音频输入
    println!("\n3. 模拟音频输入测试...");
    
    // 模拟用户说中文（应该被翻译成英文并通过虚拟麦克风输出）
    let chinese_audio_sample: Vec<AudioSample> = vec![100, 200, 300, 400, 500];
    switchboard.simulate_physical_mic_input(&chinese_audio_sample).await;
    
    // 模拟会议中的英文语音（应该被翻译成中文并通过物理耳机播放）
    let english_audio_sample: Vec<AudioSample> = vec![500, 400, 300, 200, 100];
    switchboard.simulate_system_loopback_input(&english_audio_sample).await;

    // 4. 演示语言切换
    println!("\n4. 演示语言对切换...");
    switchboard.send_control(AudioControl::SetLanguagePair("en".to_string(), "fr".to_string()))?;
    println!("已切换语言对: 英语 -> 法语");

    // 5. 运行一段时间后停止
    println!("\n5. 系统运行中... 按 Ctrl+C 停止");
    
    // 等待一小段时间以观察输出
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 6. 停止系统
    println!("\n6. 停止音频交换机...");
    switchboard.stop()?;
    
    println!("\n=== 演示完成 ===");
    println!("全双工双向翻译系统已成功演示:");
    println!("- 发送端流水线：物理麦克风 -> 翻译 -> 虚拟麦克风");
    println!("- 接收端流水线：系统环回 -> 翻译 -> 物理耳机");
    println!("- 支持实时双向语音翻译");
    println!("- 防止自翻译的逻辑隔离");

    Ok(())
}
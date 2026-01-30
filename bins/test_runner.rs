//! 测试运行器
//! 用于运行音频模拟测试

use std::path::PathBuf;
use tokio::fs;
use tokio::time::{sleep, Duration};
use real_trans::tests::audio_simulation::{AudioSimulationConfig, AudioSimulationTester};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 音频模拟测试运行器 ===\n");

    // 1. 创建测试配置
    println!("1. 创建测试配置...");
    let config = AudioSimulationConfig {
        source_input_dir: PathBuf::from("./tests/data/source_input"),
        source_output_dir: PathBuf::from("./tests/data/source_output"),
        target_input_dir: PathBuf::from("./tests/data/target_input"),
        target_output_dir: PathBuf::from("./tests/data/target_output"),
        user_language: "zh".to_string(),  // 用户说中文
        other_language: "en".to_string(), // 对方说英文
    };

    // 2. 创建测试器
    println!("2. 创建音频模拟测试器...");
    let mut tester = AudioSimulationTester::new(config).await?;
    
    // 3. 启动测试器
    println!("3. 启动音频模拟测试器...");
    tester.start().await?;

    // 4. 创建模拟音频文件
    println!("4. 创建模拟音频文件进行测试...");
    
    // 创建源输入音频文件（用户说中文）
    fs::create_dir_all(&tester.config().source_input_dir).await?;
    let source_audio_file = tester.config().source_input_dir.join("user_speaking_zh.wav");
    fs::write(&source_audio_file, b"simulated chinese audio data").await?;
    println!("   创建源输入音频文件: {:?}", source_audio_file);

    // 创建目标输入音频文件（对方说英文）
    fs::create_dir_all(&tester.config().target_input_dir).await?;
    let target_audio_file = tester.config().target_input_dir.join("other_speaking_en.wav");
    fs::write(&target_audio_file, b"simulated english audio data").await?;
    println!("   创建目标输入音频文件: {:?}", target_audio_file);

    // 5. 简单处理音频文件
    println!("5. 处理音频文件...");
    
    // 处理源输入目录中的文件
    let mut source_entries = fs::read_dir(&tester.config().source_input_dir).await?;
    while let Ok(Some(entry)) = source_entries.next_entry().await {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "wav" || ext == "pcm" || ext == "mp3") {
            println!("发现源输入音频文件: {:?}", path);
            
            // 读取音频文件内容（模拟真实的音频数据）
            let audio_data = tester.read_audio_file(&path).await.unwrap_or_else(|_| vec![0; 100]);
            
            // 将音频数据传递给翻译器（用户说话）
            {
                let app_context = tester.app_context.lock().await;
                app_context.simulate_user_speaking(&audio_data).await;
            }

            // 等待翻译结果
            sleep(Duration::from_millis(500)).await;

            // 移动处理过的音频文件到备份位置
            let backup_path = tester.config().source_input_dir.join("processed")
                .join(path.file_name().unwrap());
            fs::create_dir_all(backup_path.parent().unwrap()).await.ok();
            fs::rename(&path, &backup_path).await.ok();
        }
    }

    // 处理目标输入目录中的文件
    let mut target_entries = fs::read_dir(&tester.config().target_input_dir).await?;
    while let Ok(Some(entry)) = target_entries.next_entry().await {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "wav" || ext == "pcm" || ext == "mp3") {
            println!("发现目标输入音频文件: {:?}", path);
            
            // 读取音频文件内容（模拟真实的音频数据）
            let audio_data = tester.read_audio_file(&path).await.unwrap_or_else(|_| vec![0; 100]);
            
            // 将音频数据传递给翻译器（对方说话）
            {
                let app_context = tester.app_context.lock().await;
                app_context.simulate_other_speaking(&audio_data);
            }

            // 等待翻译结果
            sleep(Duration::from_millis(500)).await;

            // 移动处理过的音频文件到备份位置
            let backup_path = tester.config().target_input_dir.join("processed")
                .join(path.file_name().unwrap());
            fs::create_dir_all(backup_path.parent().unwrap()).await.ok();
            fs::rename(&path, &backup_path).await.ok();
        }
    }

    // 6. 等待一些时间让翻译完成
    println!("6. 等待翻译处理完成...");
    sleep(Duration::from_secs(2)).await;

    // 7. 检查输出目录
    println!("7. 检查输出目录...");
    
    // 检查源输出目录（中文翻译成英文的结果）
    let mut source_entries = fs::read_dir(&tester.config().source_output_dir).await?;
    let mut source_outputs = Vec::new();
    while let Ok(Some(entry)) = source_entries.next_entry().await {
        source_outputs.push(entry.path());
    }
    
    println!("   源输出目录文件数: {}", source_outputs.len());
    for path in &source_outputs {
        println!("     - {:?}", path);
    }

    // 检查目标输出目录（英文翻译成中文的结果）
    let mut target_entries = fs::read_dir(&tester.config().target_output_dir).await?;
    let mut target_outputs = Vec::new();
    while let Ok(Some(entry)) = target_entries.next_entry().await {
        target_outputs.push(entry.path());
    }
    
    println!("   目标输出目录文件数: {}", target_outputs.len());
    for path in &target_outputs {
        println!("     - {:?}", path);
    }

    // 8. 获取翻译结果统计
    let result_count = tester.get_result_count().await;
    println!("8. 翻译结果统计: {} 条", result_count);

    let all_results = tester.get_all_results().await;
    for (i, result) in all_results.iter().enumerate() {
        println!("   结果 {}: 方向={:?}, 原文='{}', 译文='{}'", 
                 i + 1, 
                 result.direction, 
                 result.original_text, 
                 result.translated_text);
    }

    // 9. 停止测试器
    println!("\n9. 停止音频模拟测试器...");
    tester.stop().await?;

    println!("\n=== 音频模拟测试完成 ===");
    println!("\n测试总结:");
    println!("- 源输入目录: {:?}", PathBuf::from("./tests/data/source_input"));
    println!("- 源输出目录: {:?}", PathBuf::from("./tests/data/source_output"));
    println!("- 目标输入目录: {:?}", PathBuf::from("./tests/data/target_input"));
    println!("- 目标输出目录: {:?}", PathBuf::from("./tests/data/target_output"));
    println!("- 用户语言: zh");
    println!("- 对方语言: en");
    println!("- 处理结果数: {}", result_count);

    Ok(())
}


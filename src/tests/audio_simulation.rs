//! 音频模拟测试模块
//! 用于模拟真实的音频输入/输出场景

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use crate::{
    bidirectional_translator::{BidirectionalResult, TranslationDirection},
    virtual_audio_manager::AppContext,
    AudioSample
};

/// 音频模拟测试配置
#[derive(Debug, Clone)]
pub struct AudioSimulationConfig {
    /// 源输入目录 - 存放用户语言的音频文件
    pub source_input_dir: PathBuf,
    /// 源输出目录 - 存放用户语言翻译成对方语言的结果
    pub source_output_dir: PathBuf,
    /// 目标输入目录 - 存放对方语言的音频文件
    pub target_input_dir: PathBuf,
    /// 目标输出目录 - 存放对方语言翻译成用户语言的结果
    pub target_output_dir: PathBuf,
    /// 用户语言
    pub user_language: String,
    /// 对方语言
    pub other_language: String,
}

impl Default for AudioSimulationConfig {
    fn default() -> Self {
        AudioSimulationConfig {
            source_input_dir: PathBuf::from("./tests/data/source_input"),
            source_output_dir: PathBuf::from("./tests/data/source_output"),
            target_input_dir: PathBuf::from("./tests/data/target_input"),
            target_output_dir: PathBuf::from("./tests/data/target_output"),
            user_language: "zh".to_string(),
            other_language: "en".to_string(),
        }
    }
}

/// 音频模拟测试器
pub struct AudioSimulationTester {
    /// 测试配置
    pub config: AudioSimulationConfig,
    /// 应用程序上下文
    pub app_context: Arc<tokio::sync::Mutex<AppContext>>,
    /// 翻译结果缓存
    translation_results: Arc<tokio::sync::Mutex<Vec<BidirectionalResult>>>,
}

impl AudioSimulationTester {
    /// 创建新的音频模拟测试器
    pub async fn new(config: AudioSimulationConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建必要的目录
        Self::ensure_directories(&config).await?;

        // 创建应用程序上下文
        let mut app_context = AppContext::new(&config.user_language, &config.other_language)?;
        app_context.initialize()?;

        // 创建翻译结果缓存
        let translation_results = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        // 设置翻译结果回调
        let results_clone = Arc::clone(&translation_results);
        app_context.set_translation_handler(move |result| {
            let mut results = results_clone.blocking_lock();
            results.push(result.clone());
        });

        Ok(AudioSimulationTester {
            config,
            app_context: Arc::new(tokio::sync::Mutex::new(app_context)),
            translation_results,
        })
    }

    /// 确保测试目录存在
    async fn ensure_directories(config: &AudioSimulationConfig) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&config.source_input_dir).await?;
        fs::create_dir_all(&config.source_output_dir).await?;
        fs::create_dir_all(&config.target_input_dir).await?;
        fs::create_dir_all(&config.target_output_dir).await?;
        Ok(())
    }

    /// 启动测试器
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.app_context.lock().await.start()?;
        println!("音频模拟测试器已启动");
        Ok(())
    }

    /// 停止测试器
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.app_context.lock().await.stop()?;
        println!("音频模拟测试器已停止");
        Ok(())
    }

    /// 监控源输入目录并处理音频文件
    pub async fn monitor_source_input(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始监控源输入目录: {:?}", self.config.source_input_dir);

        // 简单的轮询监控（实际实现中可以使用文件系统监听器）
        loop {
            // 查找源输入目录中的音频文件
            let mut entries = fs::read_dir(&self.config.source_input_dir).await?;
            
            let mut has_files = false;
            let mut entries_vec = Vec::new();
            
            // 收集所有音频文件
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "wav" || ext == "pcm" || ext == "mp3") {
                    entries_vec.push(path);
                    has_files = true;
                }
            }

            // 处理每个音频文件
            for audio_file_path in entries_vec {
                println!("发现源输入音频文件: {:?}", audio_file_path);
                
                // 读取音频文件内容（模拟真实的音频数据）
                let audio_data = match self.read_audio_file(&audio_file_path).await {
                    Ok(data) => data,
                    Err(_) => continue, // 如果读取失败则跳过这个文件
                };
                
                // 将音频数据传递给翻译器（用户说话）
                {
                    let app_context = self.app_context.lock().await;
                    app_context.simulate_user_speaking(&audio_data).await;
                }

                // 等待翻译结果
                sleep(Duration::from_millis(1000)).await;

                // 获取最新的翻译结果
                {
                    let results = self.translation_results.lock().await;
                    let latest_results: Vec<_> = results.iter()
                        .filter(|r| r.direction == TranslationDirection::UserToOther)
                        .cloned()
                        .collect();

                    for result in latest_results {
                        // 将翻译结果保存到源输出目录
                        let output_filename = format!("translated_{}.txt", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                        let output_path = self.config.source_output_dir.join(output_filename);
                        
                        tokio::fs::write(
                            &output_path,
                            format!(
                                "原文: {}\n译文: {}\n时间: {:?}",
                                result.original_text,
                                result.translated_text,
                                result.timestamp
                            )
                        ).await?;
                        
                        println!("已保存翻译结果到: {:?}", output_path);
                    }
                }

                // 移动处理过的音频文件到备份位置或删除
                let backup_path = self.config.source_input_dir.join("processed")
                    .join(audio_file_path.file_name().unwrap());
                tokio::fs::create_dir_all(backup_path.parent().unwrap()).await.ok();
                tokio::fs::rename(&audio_file_path, &backup_path).await.ok();
            }

            if has_files {
                println!("处理完源输入目录中的音频文件");
            }

            // 等待一段时间再检查
            sleep(Duration::from_millis(5000)).await;
        }
    }

    /// 监控目标输入目录并处理音频文件
    pub async fn monitor_target_input(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始监控目标输入目录: {:?}", self.config.target_input_dir);

        // 简单的轮询监控（实际实现中可以使用文件系统监听器）
        loop {
            // 查找目标输入目录中的音频文件
            let mut entries = fs::read_dir(&self.config.target_input_dir).await?;
            
            let mut has_files = false;
            let mut entries_vec = Vec::new();
            
            // 收集所有音频文件
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "wav" || ext == "pcm" || ext == "mp3") {
                    entries_vec.push(path);
                    has_files = true;
                }
            }

            // 处理每个音频文件
            for audio_file_path in entries_vec {
                println!("发现目标输入音频文件: {:?}", audio_file_path);
                
                // 读取音频文件内容（模拟真实的音频数据）
                let audio_data = match self.read_audio_file(&audio_file_path).await {
                    Ok(data) => data,
                    Err(_) => continue, // 如果读取失败则跳过这个文件
                };
                
                // 将音频数据传递给翻译器（对方说话）
                {
                    let app_context = self.app_context.lock().await;
                    app_context.simulate_other_speaking(&audio_data);
                }

                // 等待翻译结果
                sleep(Duration::from_millis(1000)).await;

                // 获取最新的翻译结果
                {
                    let results = self.translation_results.lock().await;
                    let latest_results: Vec<_> = results.iter()
                        .filter(|r| r.direction == TranslationDirection::OtherToUser)
                        .cloned()
                        .collect();

                    for result in latest_results {
                        // 将翻译结果保存到目标输出目录
                        let output_filename = format!("translated_{}.txt", 
                            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                        let output_path = self.config.target_output_dir.join(output_filename);
                        
                        tokio::fs::write(
                            &output_path,
                            format!(
                                "原文: {}\n译文: {}\n时间: {:?}",
                                result.original_text,
                                result.translated_text,
                                result.timestamp
                            )
                        ).await?;
                        
                        println!("已保存翻译结果到: {:?}", output_path);
                    }
                }

                // 移动处理过的音频文件到备份位置或删除
                let backup_path = self.config.target_input_dir.join("processed")
                    .join(audio_file_path.file_name().unwrap());
                tokio::fs::create_dir_all(backup_path.parent().unwrap()).await.ok();
                tokio::fs::rename(&audio_file_path, &backup_path).await.ok();
            }

            if has_files {
                println!("处理完目标输入目录中的音频文件");
            }

            // 等待一段时间再检查
            sleep(Duration::from_millis(5000)).await;
        }
    }

    /// 获取翻译结果数量
    pub async fn get_result_count(&self) -> usize {
        self.translation_results.lock().await.len()
    }

    /// 获取所有翻译结果
    pub async fn get_all_results(&self) -> Vec<BidirectionalResult> {
        self.translation_results.lock().await.clone()
    }

    /// 运行完整的音频模拟测试
    pub async fn run_full_simulation(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("开始运行完整的音频模拟测试...");

        // 启动两个监控任务
        let source_monitor = self.monitor_source_input();
        let target_monitor = self.monitor_target_input();

        // 并行运行监控任务
        tokio::select! {
            result = source_monitor => {
                println!("源输入监控结束: {:?}", result);
            }
            result = target_monitor => {
                println!("目标输入监控结束: {:?}", result);
            }
        }

        Ok(())
    }

    /// 获取配置的引用
    pub fn config(&self) -> &AudioSimulationConfig {
        &self.config
    }

    /// 读取音频文件并返回模拟的音频数据
    pub async fn read_audio_file(&self, path: &PathBuf) -> Result<Vec<AudioSample>, Box<dyn std::error::Error + Send + Sync>> {
        // 这里应该实现实际的音频文件读取逻辑
        // 为了演示目的，我们生成模拟的音频数据
        println!("读取音频文件: {:?}", path);
        
        // 生成模拟音频数据
        let mut audio_data = Vec::new();
        for i in 0..1000 {
            // 生成简单的模拟音频样本
            let sample = (i % 1000) as AudioSample;
            audio_data.push(sample);
        }
        
        Ok(audio_data)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_audio_simulation_setup() {
        let config = AudioSimulationConfig {
            source_input_dir: PathBuf::from("./tests/data/test_source_input"),
            source_output_dir: PathBuf::from("./tests/data/test_source_output"),
            target_input_dir: PathBuf::from("./tests/data/test_target_input"),
            target_output_dir: PathBuf::from("./tests/data/test_target_output"),
            user_language: "zh".to_string(),
            other_language: "en".to_string(),
        };

        let tester = AudioSimulationTester::new(config).await;
        assert!(tester.is_ok());

        let mut tester = tester.unwrap();
        assert!(tester.start().await.is_ok());
        assert!(tester.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_read_audio_file() {
        let config = AudioSimulationConfig::default();
        let tester = AudioSimulationTester::new(config).await.unwrap();

        // 创建一个临时音频文件
        let temp_file = PathBuf::from("./tests/data/temp_test.wav");
        tokio::fs::write(&temp_file, b"fake wav data").await.unwrap();

        let audio_data = tester.read_audio_file(&temp_file).await;
        assert!(audio_data.is_ok());
        assert!(!audio_data.unwrap().is_empty());

        // 清理临时文件
        tokio::fs::remove_file(temp_file).await.ok();
    }
}
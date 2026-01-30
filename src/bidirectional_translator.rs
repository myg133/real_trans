//! 双向实时翻译器
//! 实现用户语言和对方语言之间的双向实时翻译

use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use std::time::Instant;
use tokio::sync::oneshot;
use crate::{
    engine::translation_pipeline::{TranslationPipeline, TranslationResult, TranslationCallback},
    io::audio_capture::AudioCapture,
    AudioSample, SAMPLES_PER_FRAME
};

/// 语言对结构
#[derive(Debug, Clone)]
pub struct LanguagePair {
    pub source: String,  // 用户语言
    pub target: String,  // 对方语言
}

/// 双向翻译结果
#[derive(Debug, Clone)]
pub struct BidirectionalResult {
    pub original_text: String,
    pub translated_text: String,
    pub direction: TranslationDirection, // 翻译方向
    pub timestamp: Instant,
}

/// 翻译方向
#[derive(Debug, Clone, PartialEq)]
pub enum TranslationDirection {
    UserToOther,  // 用户语言 -> 对方语言
    OtherToUser,  // 对方语言 -> 用户语言
}

/// 双向翻译器
pub struct BidirectionalTranslator {
    // 用户到对方的翻译流水线
    user_to_other_pipeline: Arc<Mutex<TranslationPipeline>>,
    // 对方到用户的翻译流水线
    other_to_user_pipeline: Arc<Mutex<TranslationPipeline>>,
    // 音频捕获器
    audio_capture: Arc<Mutex<AudioCapture>>,
    // 回调函数
    result_callback: Option<Arc<dyn Fn(&BidirectionalResult) + Send + Sync>>,
    // 当前语言对
    current_pair: Arc<Mutex<LanguagePair>>,
    // 是否正在运行
    running: std::sync::atomic::AtomicBool,
    // 当前翻译方向
    current_direction: Arc<Mutex<TranslationDirection>>,
}

impl BidirectionalTranslator {
    /// 创建新的双向翻译器
    pub fn new(user_lang: &str, other_lang: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建用户到对方的翻译流水线 (例如: 中文 -> 英语)
        let mut user_to_other_pipeline = TranslationPipeline::new(
            "./models/whisper-tiny.bin".to_string(),
            "./models/qwen2.5-0.5b.bin".to_string(),
        );
        
        // 创建对方到用户的翻译流水线 (例如: 英语 -> 中文)
        let mut other_to_user_pipeline = TranslationPipeline::new(
            "./models/whisper-tiny.bin".to_string(),
            "./models/qwen2.5-0.5b.bin".to_string(),
        );

        // 设置语言对
        user_to_other_pipeline.set_source_language(user_lang);
        user_to_other_pipeline.set_target_language(other_lang);
        
        other_to_user_pipeline.set_source_language(other_lang);
        other_to_user_pipeline.set_target_language(user_lang);

        // 初始化流水线
        user_to_other_pipeline.initialize()?;
        other_to_user_pipeline.initialize()?;

        // 创建音频捕获器
        let mut audio_capture = AudioCapture::new();
        audio_capture.initialize()?;

        Ok(BidirectionalTranslator {
            user_to_other_pipeline: Arc::new(Mutex::new(user_to_other_pipeline)),
            other_to_user_pipeline: Arc::new(Mutex::new(other_to_user_pipeline)),
            audio_capture: Arc::new(Mutex::new(audio_capture)),
            result_callback: None,
            current_pair: Arc::new(Mutex::new(LanguagePair {
                source: user_lang.to_string(),
                target: other_lang.to_string(),
            })),
            running: std::sync::atomic::AtomicBool::new(false),
            current_direction: Arc::new(Mutex::new(TranslationDirection::UserToOther)),
        })
    }

    /// 设置结果回调函数
    pub fn set_result_callback(&mut self, callback: impl Fn(&BidirectionalResult) + Send + Sync + 'static) {
        self.result_callback = Some(Arc::new(callback));

        // 为两个流水线设置回调
        let callback_clone = self.result_callback.clone();
        let direction = self.current_direction.clone();
        
        self.user_to_other_pipeline.lock().unwrap().set_translation_callback(move |result| {
            if let Some(ref cb) = callback_clone {
                let bidir_result = BidirectionalResult {
                    original_text: result.original_text.clone(),
                    translated_text: result.translated_text.clone(),
                    direction: TranslationDirection::UserToOther,
                    timestamp: result.timestamp,
                };
                cb(&bidir_result);
            }
        });

        let callback_clone = self.result_callback.clone();
        self.other_to_user_pipeline.lock().unwrap().set_translation_callback(move |result| {
            if let Some(ref cb) = callback_clone {
                let bidir_result = BidirectionalResult {
                    original_text: result.original_text.clone(),
                    translated_text: result.translated_text.clone(),
                    direction: TranslationDirection::OtherToUser,
                    timestamp: result.timestamp,
                };
                cb(&bidir_result);
            }
        });
    }

    /// 启动双向翻译器
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        // 启动两个翻译流水线
        self.user_to_other_pipeline.lock().unwrap().start()?;
        self.other_to_user_pipeline.lock().unwrap().start()?;

        // 设置音频数据回调，处理捕获到的音频
        let user_to_other_pipeline = self.user_to_other_pipeline.clone();
        let other_to_user_pipeline = self.other_to_user_pipeline.clone();
        let current_direction = self.current_direction.clone();

        self.audio_capture.lock().unwrap().set_data_callback(Box::new(move |audio_data, is_final| {
            // 根据当前方向决定使用哪个流水线处理音频
            let direction = current_direction.lock().unwrap().clone();
            
            match direction {
                TranslationDirection::UserToOther => {
                    // 用户说话，转换为目标语言
                    let mut pipeline = user_to_other_pipeline.lock().unwrap();
                    pipeline.process_audio(audio_data);
                }
                TranslationDirection::OtherToUser => {
                    // 对方说话，转换为用户语言
                    let mut pipeline = other_to_user_pipeline.lock().unwrap();
                    pipeline.process_audio(audio_data);
                }
            }
        }));

        // 启动音频捕获
        self.audio_capture.lock().unwrap().start()?;

        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 停止双向翻译器
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }

        // 停止音频捕获
        self.audio_capture.lock().unwrap().stop()?;

        // 停止两个翻译流水线
        self.user_to_other_pipeline.lock().unwrap().stop()?;
        self.other_to_user_pipeline.lock().unwrap().stop()?;

        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// 切换当前翻译方向
    pub fn switch_direction(&self) {
        let mut current_dir = self.current_direction.lock().unwrap();
        *current_dir = match *current_dir {
            TranslationDirection::UserToOther => TranslationDirection::OtherToUser,
            TranslationDirection::OtherToUser => TranslationDirection::UserToOther,
        };
    }

    /// 更新语言对
    pub fn update_language_pair(&mut self, user_lang: &str, other_lang: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 更新当前语言对
        {
            let mut pair = self.current_pair.lock().unwrap();
            pair.source = user_lang.to_string();
            pair.target = other_lang.to_string();
        }

        // 更新两个流水线的语言设置
        {
            let mut pipeline = self.user_to_other_pipeline.lock().unwrap();
            pipeline.set_source_language(user_lang);
            pipeline.set_target_language(other_lang);
        }

        {
            let mut pipeline = self.other_to_user_pipeline.lock().unwrap();
            pipeline.set_source_language(other_lang);
            pipeline.set_target_language(user_lang);
        }

        Ok(())
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// 获取当前语言对
    pub fn get_current_language_pair(&self) -> LanguagePair {
        self.current_pair.lock().unwrap().clone()
    }

    /// 处理传入的音频数据（例如，从虚拟音频设备接收）
    pub fn handle_incoming_audio(&self, audio_data: &[AudioSample], is_user_speaking: bool) {
        if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }

        // 根据说话者身份决定使用哪个流水线
        if is_user_speaking {
            // 用户说话，转换为目标语言
            let mut pipeline = self.user_to_other_pipeline.lock().unwrap();
            pipeline.process_audio(audio_data);
        } else {
            // 对方说话，转换为用户语言
            let mut pipeline = self.other_to_user_pipeline.lock().unwrap();
            pipeline.process_audio(audio_data);
        }
    }

    /// 处理发送端音频（用户说话，翻译成对方语言）
    pub async fn handle_outbound_audio(&self, audio_data: &[AudioSample]) {
        // 设置当前方向为用户到对方
        {
            let mut dir = self.current_direction.lock().unwrap();
            *dir = TranslationDirection::UserToOther;
        }

        // 通过用户到对方的翻译流水线处理音频
        let mut pipeline = self.user_to_other_pipeline.lock().unwrap();
        pipeline.process_audio(audio_data);
    }

    /// 处理接收端音频（对方说话，翻译成用户语言）
    pub async fn handle_inbound_audio(&self, audio_data: &[AudioSample]) {
        // 设置当前方向为对方到用户
        {
            let mut dir = self.current_direction.lock().unwrap();
            *dir = TranslationDirection::OtherToUser;
        }

        // 通过对方到用户的翻译流水线处理音频
        let mut pipeline = self.other_to_user_pipeline.lock().unwrap();
        pipeline.process_audio(audio_data);
    }

    /// 模拟用户说话
    pub async fn simulate_user_speaking(&self, audio_data: &[AudioSample]) {
        // 设置当前方向为用户到对方
        {
            let mut dir = self.current_direction.lock().unwrap();
            *dir = TranslationDirection::UserToOther;
        }

        // 通过音频捕获器处理音频
        self.audio_capture.lock().unwrap().simulate_audio_input(audio_data).await;
    }

    /// 模拟对方说话
    pub async fn simulate_other_speaking(&self, audio_data: &[AudioSample]) {
        // 设置当前方向为对方到用户
        {
            let mut dir = self.current_direction.lock().unwrap();
            *dir = TranslationDirection::OtherToUser;
        }

        // 通过音频捕获器处理音频
        self.audio_capture.lock().unwrap().simulate_audio_input(audio_data).await;
    }

    /// 切换到用户模式
    pub fn switch_to_user_mode(&self) {
        let mut dir = self.current_direction.lock().unwrap();
        *dir = TranslationDirection::UserToOther;
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> TranslationStats {
        TranslationStats {
            total_processed: 0,      // 在实际实现中，应跟踪处理的数量
            successful_translations: 0,
            error_count: 0,
            avg_latency_ms: 0.0,
        }
    }
}

/// 翻译统计信息
#[derive(Debug, Clone)]
pub struct TranslationStats {
    pub total_processed: u64,
    pub successful_translations: u64,
    pub error_count: u64,
    pub avg_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_translator_creation() {
        let translator = BidirectionalTranslator::new("zh", "en");
        assert!(translator.is_ok());
    }

    #[tokio::test]
    async fn test_language_pair_update() {
        let mut translator = BidirectionalTranslator::new("zh", "en").unwrap();
        
        // 检查初始语言对
        let initial_pair = translator.get_current_language_pair();
        assert_eq!(initial_pair.source, "zh");
        assert_eq!(initial_pair.target, "en");

        // 更新语言对
        translator.update_language_pair("en", "fr").unwrap();
        
        // 检查更新后的语言对
        let updated_pair = translator.get_current_language_pair();
        assert_eq!(updated_pair.source, "en");
        assert_eq!(updated_pair.target, "fr");
    }
}
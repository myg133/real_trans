//! 实时翻译流水线模块
//! 协调VAD、ASR和MT模块，实现端到端的实时语音翻译

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use crate::engine::asr::Asr;
use crate::engine::mt::Mt;
use crate::engine::vad::Vad;
use crate::{AudioSample, SAMPLES_PER_FRAME};

/// 翻译结果结构
#[derive(Debug, Clone)]
pub struct TranslationResult {
    pub original_text: String,              // 原始文本（ASR结果）
    pub translated_text: String,            // 翻译文本（MT结果）
    pub asr_confidence: f32,               // ASR置信度
    pub mt_confidence: f32,                // MT置信度
    pub is_partial: bool,                  // 是否为部分结果
    pub is_final: bool,                    // 是否为最终结果
    pub timestamp: Instant,                // 时间戳
}

/// 翻译结果回调函数类型
pub type TranslationCallback = Arc<dyn Fn(&TranslationResult) + Send + Sync>;

/// 翻译流水线结构
pub struct TranslationPipeline {
    asr: Arc<Mutex<Asr>>,
    mt: Arc<Mutex<Mt>>,
    vad: Arc<Mutex<Vad>>,
    translation_callback: Option<TranslationCallback>,
    running: AtomicBool,
    source_language: Arc<Mutex<String>>,
    target_language: Arc<Mutex<String>>,
    // pending_translations: Arc<Mutex<Vec<String>>>,
}

impl TranslationPipeline {
    /// 创建新的翻译流水线实例
    pub fn new(asr_model_path: String, mt_model_path: String) -> Self {
        let asr = Arc::new(Mutex::new(Asr::new(asr_model_path, "whisper-tiny".to_string())));
        let mt = Arc::new(Mutex::new(Mt::new(mt_model_path, "qwen2.5-0.5b".to_string())));
        let vad = Arc::new(Mutex::new(Vad::new(crate::SAMPLE_RATE, 30)));

        TranslationPipeline {
            asr,
            mt,
            vad,
            translation_callback: None,
            running: AtomicBool::new(false),
            source_language: Arc::new(Mutex::new("zh".to_string())),
            target_language: Arc::new(Mutex::new("en".to_string())),
            // pending_translations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 设置翻译结果回调函数
    pub fn set_translation_callback(&mut self, callback: impl Fn(&TranslationResult) + Send + Sync + 'static) {
        self.translation_callback = Some(Arc::new(callback));
    }

    /// 初始化翻译流水线
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 初始化ASR和MT模块
        {
            let mut asr = self.asr.lock().unwrap();
            asr.initialize()?;
        }

        {
            let mut mt = self.mt.lock().unwrap();
            mt.initialize()?;
        }

        // 设置内部回调
        let asr_clone = Arc::clone(&self.asr);
        let pipeline_self = Arc::new(Mutex::new(self as *mut TranslationPipeline));
        
        // 注意：这里需要使用更合适的方式来设置回调，避免循环引用
        // 在实际实现中，我们会使用事件驱动的方式

        Ok(())
    }

    /// 处理音频数据
    pub fn process_audio(&mut self, audio_data: &[AudioSample]) -> bool {
        if !self.running.load(Ordering::SeqCst) {
            return false;
        }

        // 使用ASR处理音频数据
        let asr_results = {
            let mut asr = self.asr.lock().unwrap();
            asr.process_audio(audio_data)
        };

        // 处理ASR结果
        for result in asr_results {
            self.on_asr_result(result);
        }

        true
    }

    /// 处理音频帧
    pub fn process_frame(&mut self, audio_frame: &[AudioSample]) -> bool {
        if !self.running.load(Ordering::SeqCst) {
            return false;
        }

        // 使用ASR处理音频帧
        let asr_result = {
            let mut asr = self.asr.lock().unwrap();
            asr.process_frame(audio_frame)
        };

        // 处理ASR结果
        self.on_asr_result(asr_result);

        true
    }

    /// 启动翻译流水线
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        // 检查ASR和MT模块是否已初始化
        {
            let asr = self.asr.lock().unwrap();
            if !asr.is_initialized() {
                return Err("ASR module not initialized".into());
            }
        }

        {
            let mt = self.mt.lock().unwrap();
            if !mt.is_initialized() {
                return Err("MT module not initialized".into());
            }
        }

        self.running.store(true, Ordering::SeqCst);

        // 重置ASR和MT模块
        {
            let mut asr = self.asr.lock().unwrap();
            asr.reset();
        }
        {
            let mut mt = self.mt.lock().unwrap();
            mt.reset();
        }

        Ok(())
    }

    /// 停止翻译流水线
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.running.store(false, Ordering::SeqCst);

        // 停止ASR和MT模块
        {
            let mut asr = self.asr.lock().unwrap();
            asr.reset();
        }
        {
            let mut mt = self.mt.lock().unwrap();
            mt.reset();
        }

        Ok(())
    }

    /// 重置流水线状态
    pub fn reset(&mut self) {
        {
            let mut asr = self.asr.lock().unwrap();
            asr.reset();
        }
        {
            let mut mt = self.mt.lock().unwrap();
            mt.reset();
        }
    }

    /// 设置源语言
    pub fn set_source_language(&mut self, lang: &str) {
        *self.source_language.lock().unwrap() = lang.to_string();
        {
            let mut asr = self.asr.lock().unwrap();
            asr.set_language(lang.to_string());
        }
    }

    /// 设置目标语言
    pub fn set_target_language(&mut self, lang: &str) {
        *self.target_language.lock().unwrap() = lang.to_string();
        // 在实际实现中，可能需要通知MT模块更新目标语言
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 内部ASR结果处理函数
    fn on_asr_result(&self, result: crate::engine::asr::AsrResult) {
        if !result.is_final {
            // 中间结果，可以发送给UI进行实时显示
            if let Some(ref callback) = self.translation_callback {
                let trans_result = TranslationResult {
                    original_text: result.text,
                    translated_text: String::new(), // 翻译结果暂无
                    asr_confidence: result.confidence,
                    mt_confidence: 0.0,
                    is_partial: true,
                    is_final: false,
                    timestamp: result.timestamp,
                };
                
                callback(&trans_result);
            }
            return;
        }

        // 最终ASR结果，提交给MT模块进行翻译
        let source_text = result.text.clone();
        let source_lang = self.source_language.lock().unwrap().clone();
        let target_lang = self.target_language.lock().unwrap().clone();
        
        // 创建克隆以在线程间安全传递
        let mt_clone = Arc::clone(&self.mt);
        let callback_opt = self.translation_callback.clone();
        
        // 异步执行翻译，避免阻塞ASR处理
        tokio::spawn(async move {
            let mt_result = {
                let mut mt = mt_clone.lock().unwrap();
                mt.translate(&source_text, Some(&source_lang), Some(&target_lang))
            };
            
            // 合并结果并发送回调
            if let Some(callback) = callback_opt {
                let trans_result = TranslationResult {
                    original_text: mt_result.source_text,
                    translated_text: mt_result.translated_text,
                    asr_confidence: 0.9, // 使用实际的ASR置信度
                    mt_confidence: mt_result.confidence,
                    is_partial: false,
                    is_final: true,
                    timestamp: Instant::now(),
                };
                
                callback(&trans_result);
            }
        });
    }

    /// 内部MT结果处理函数
    fn _on_mt_result(&self, _result: crate::engine::mt::MtResult) {
        // MT结果处理 - 在实际实现中，这可能由异步线程调用
        // 根据MT结果更新UI或其他组件
    }

    /// 内部VAD结果处理函数
    fn _on_vad_result(&self, _result: crate::engine::vad::VadResult) {
        // VAD结果处理 - 在实际实现中，这将连接VAD模块
        // 根据VAD结果控制ASR模块的激活状态
    }
}

impl Drop for TranslationPipeline {
    fn drop(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_pipeline_creation() {
        let pipeline = TranslationPipeline::new(
            "./models/whisper-tiny.bin".to_string(),
            "./models/qwen2.5-0.5b.bin".to_string(),
        );
        assert!(!pipeline.is_running());
    }
}
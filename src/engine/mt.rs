//! 机器翻译（Machine Translation）模块
//! 封装了文本翻译的功能，支持多种语言对的翻译

use std::collections::VecDeque;
use std::time::Instant;
use std::sync::{Arc, Mutex};

/// 翻译结果结构
#[derive(Debug, Clone)]
pub struct MtResult {
    pub source_text: String,                // 源文本
    pub translated_text: String,            // 翻译文本
    pub source_language: String,            // 源语言
    pub target_language: String,            // 目标语言
    pub confidence: f32,                    // 置信度 (0.0-1.0)
    pub is_success: bool,                   // 是否翻译成功
    pub timestamp: Instant,                 // 时间戳
}

/// 翻译结果回调函数类型
pub type ResultCallback = Box<dyn Fn(&MtResult) + Send>;

/// 翻译上下文结构
#[derive(Debug, Clone)]
pub struct Context {
    pub source_text: String,
    pub translated_text: String,
    pub timestamp: Instant,
}

/// 机器翻译器
pub struct Mt {
    model_path: String,
    model_type: String,
    default_source_lang: String,
    default_target_lang: String,
    initialized: bool,
    model_loaded: bool,
    result_callback: Option<ResultCallback>,
    context_history: Arc<Mutex<VecDeque<Context>>>,
    max_context_size: usize,
    parameters: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl Mt {
    /// 创建新的MT实例
    pub fn new(model_path: String, model_type: String) -> Self {
        Mt {
            model_path,
            model_type,
            default_source_lang: "auto".to_string(),
            default_target_lang: "en".to_string(),
            initialized: false,
            model_loaded: false,
            result_callback: None,
            context_history: Arc::new(Mutex::new(VecDeque::new())),
            max_context_size: 3,  // 默认保存3个上下文
            parameters: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// 设置翻译结果回调函数
    pub fn set_result_callback(&mut self, callback: ResultCallback) {
        self.result_callback = Some(callback);
    }

    /// 初始化MT引擎
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }

        // 检查模型文件是否存在
        if self.model_path.is_empty() {
            return Err("Error: Model path is empty".into());
        }

        // 模拟模型加载过程
        println!("Loading MT model: {} from {}", self.model_type, self.model_path);
        
        // 实际的模型加载逻辑会在这里实现
        // 例如：使用llm-rs加载Qwen或Llama模型
        
        self.model_loaded = true;
        self.initialized = true;
        
        println!("MT model loaded successfully");
        Ok(())
    }

    /// 执行翻译
    pub fn translate(
        &mut self,
        source_text: &str,
        source_lang: Option<&str>,
        target_lang: Option<&str>,
    ) -> MtResult {
        let source_lang = source_lang.unwrap_or(&self.default_source_lang);
        let target_lang = target_lang.unwrap_or(&self.default_target_lang);

        let mut result = MtResult {
            source_text: source_text.to_string(),
            translated_text: format!("模拟翻译结果: {}", source_text), // 实际应用中这里会是真实的翻译文本
            source_language: source_lang.to_string(),
            target_language: target_lang.to_string(),
            confidence: 0.85,
            is_success: true,
            timestamp: Instant::now(),
        };

        // 添加到上下文历史
        let context = Context {
            source_text: source_text.to_string(),
            translated_text: result.translated_text.clone(),
            timestamp: Instant::now(),
        };
        self.add_context(context);

        // 如果设置了回调，则调用回调
        if let Some(ref callback) = self.result_callback {
            callback(&result);
        }

        result
    }

    /// 批量翻译
    pub fn batch_translate(
        &mut self,
        texts: &[String],
        source_lang: Option<&str>,
        target_lang: Option<&str>,
    ) -> Vec<MtResult> {
        texts
            .iter()
            .map(|text| self.translate(text, source_lang, target_lang))
            .collect()
    }

    /// 添加翻译上下文
    fn add_context(&mut self, context: Context) {
        let mut history = self.context_history.lock().unwrap();
        history.push_back(context);

        // 限制上下文历史大小
        while history.len() > self.max_context_size {
            history.pop_front();
        }
    }

    /// 获取最近的翻译上下文
    pub fn get_context_history(&self, count: usize) -> Vec<Context> {
        let history = self.context_history.lock().unwrap();
        let n = std::cmp::min(history.len(), count);
        let start_index = history.len().saturating_sub(n);

        history
            .iter()
            .skip(start_index)
            .cloned()
            .collect()
    }

    /// 清空翻译上下文
    pub fn clear_context(&mut self) {
        self.context_history.lock().unwrap().clear();
    }

    /// 设置翻译参数
    pub fn set_parameter(&self, key: String, value: String) {
        let mut params = self.parameters.lock().unwrap();
        params.insert(key, value);
    }

    /// 检查是否支持特定语言对
    pub fn is_language_pair_supported(&self, _source_lang: &str, _target_lang: &str) -> bool {
        // 在实际实现中，这里会检查模型是否支持特定语言对
        // 目前简单返回true
        true
    }

    /// 重置MT状态
    pub fn reset(&mut self) {
        self.clear_context();
        // 重置其他状态（如有需要）
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 检查模型是否已加载
    pub fn is_model_loaded(&self) -> bool {
        self.model_loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mt_creation() {
        let mt = Mt::new("./models/qwen2.5-0.5b.bin".to_string(), "qwen2.5-0.5b".to_string());
        assert_eq!(mt.model_type, "qwen2.5-0.5b");
    }

    #[test]
    fn test_mt_translate() {
        let mut mt = Mt::new("./models/test.bin".to_string(), "test".to_string());
        mt.initialize().unwrap();
        
        let result = mt.translate("Hello", Some("en"), Some("zh"));
        assert!(result.is_success);
        assert_eq!(result.source_language, "en");
        assert_eq!(result.target_language, "zh");
    }
}
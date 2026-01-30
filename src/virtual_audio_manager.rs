//! 虚拟音频设备管理器
//! 管理虚拟音频输入和输出设备，用于在线会议场景

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::io::audio_device::{AudioDevice, MockAudioDevice, AudioSample};
use crate::io::virtual_audio_device::VirtualAudioDevice;
use crate::bidirectional_translator::{BidirectionalTranslator, BidirectionalResult, TranslationDirection};

/// 虚拟音频设备类型
#[derive(Debug, Clone, PartialEq)]
pub enum VirtualDeviceType {
    Input,   // 虚拟输入设备
    Output,  // 虚拟输出设备
}

/// 虚拟音频设备管理器
pub struct VirtualAudioManager {
    /// 虚拟输入设备
    virtual_input_device: Option<Box<dyn AudioDevice>>,
    /// 虚拟输出设备
    virtual_output_device: Option<Box<dyn AudioDevice>>,
    /// 设备注册表
    device_registry: HashMap<String, VirtualDeviceType>,
    /// 双向翻译器
    translator: Arc<Mutex<BidirectionalTranslator>>,
    /// 输入音频数据回调
    input_callback: Option<Box<dyn Fn(&[AudioSample]) + Send + Sync>>,
    /// 输出音频数据回调
    output_callback: Option<Box<dyn Fn(&[AudioSample]) + Send + Sync>>,
    /// 是否已激活
    active: bool,
}

impl VirtualAudioManager {
    /// 创建新的虚拟音频管理器
    pub fn new(translator: Arc<Mutex<BidirectionalTranslator>>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(VirtualAudioManager {
            virtual_input_device: Some(Box::new(VirtualAudioDevice::new("virtual_input", "virtual_output", 16000, 1))),
            virtual_output_device: Some(Box::new(VirtualAudioDevice::new("virtual_input", "virtual_output", 16000, 1))),
            device_registry: HashMap::new(),
            translator,
            input_callback: None,
            output_callback: None,
            active: false,
        })
    }

    /// 注册虚拟音频设备
    pub fn register_virtual_devices(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 注册虚拟输入设备
        self.device_registry.insert("virtual_input".to_string(), VirtualDeviceType::Input);
        
        // 注册虚拟输出设备
        self.device_registry.insert("virtual_output".to_string(), VirtualDeviceType::Output);
        
        println!("Registered virtual input device: virtual_input");
        println!("Registered virtual output device: virtual_output");
        
        Ok(())
    }

    /// 设置输入音频回调（当用户说话时）
    pub fn set_input_callback(&mut self, callback: Box<dyn Fn(&[AudioSample]) + Send + Sync>) {
        self.input_callback = Some(callback);
    }

    /// 设置输出音频回调（当需要播放翻译后的音频时）
    pub fn set_output_callback(&mut self, callback: Box<dyn Fn(&[AudioSample]) + Send + Sync>) {
        self.output_callback = Some(callback);
    }

    /// 启动虚拟音频管理器
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.active {
            return Ok(());
        }

        // 注册虚拟设备
        self.register_virtual_devices()?;

        // 设置输入回调，处理原始音频
        if let Some(ref mut input_device) = self.virtual_input_device {
            input_device.open_input_stream(Some("virtual_input".to_string()), Box::new({
                let translator = Arc::clone(&self.translator);
                let input_callback = self.input_callback.take(); // Take ownership
                
                move |audio_data| {
                    // 将原始音频数据传递给翻译器（用户说话）
                    let translator_clone = Arc::clone(&translator);
                    let audio_vec = audio_data.to_vec();
                    
                    // 在后台任务中处理音频
                    tokio::spawn(async move {
                        translator_clone.lock().unwrap().handle_incoming_audio(&audio_vec, true);
                    });
                    
                    // 如果有额外的输入回调，则调用
                    if let Some(ref cb) = input_callback {
                        cb(audio_data);
                    }
                }
            }))?;
            
            input_device.start_recording()?;
        }

        // 设置输出回调，用于播放翻译后的音频
        if let Some(ref mut output_device) = self.virtual_output_device {
            output_device.open_output_stream(Some("virtual_output".to_string()))?;
        }

        self.active = true;
        Ok(())
    }

    /// 停止虚拟音频管理器
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.active {
            return Ok(());
        }

        // 停止输入设备
        if let Some(ref mut input_device) = self.virtual_input_device {
            input_device.stop_recording()?;
            input_device.close_input_stream()?;
        }

        // 停止输出设备
        if let Some(ref mut output_device) = self.virtual_output_device {
            output_device.close_output_stream()?;
        }

        self.active = false;
        Ok(())
    }

    /// 播放翻译后的音频
    pub fn play_translated_audio(&mut self, audio_data: &[AudioSample]) -> Result<usize, Box<dyn std::error::Error>> {
        if !self.active {
            return Err("Virtual audio manager not active".into());
        }

        // 通过输出设备播放音频
        if let Some(ref mut output_device) = self.virtual_output_device {
            output_device.play_audio(audio_data)
        } else {
            Ok(0)
        }
    }

    /// 获取虚拟输入设备ID
    pub fn get_virtual_input_device_id(&self) -> String {
        "virtual_input".to_string()
    }

    /// 获取虚拟输出设备ID
    pub fn get_virtual_output_device_id(&self) -> String {
        "virtual_output".to_string()
    }

    /// 检查是否已激活
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 模拟接收到对方说话的音频（在线会议中对方的声音）
    pub fn simulate_receive_other_audio(&self, audio_data: &[AudioSample]) {
        // 将对方的音频传递给翻译器（对方说话，需要翻译成用户语言）
        let translator_clone = Arc::clone(&self.translator);
        let audio_vec = audio_data.to_vec();
        
        tokio::spawn(async move {
            translator_clone.lock().unwrap().handle_incoming_audio(&audio_vec, false);
        });
    }

    /// 添加翻译结果处理器
    pub fn set_translation_handler<F>(&mut self, handler: F)
    where
        F: Fn(&BidirectionalResult) + Send + Sync + 'static,
    {
        // 设置翻译器的结果回调
        self.translator.lock().unwrap().set_result_callback(handler);
    }
}

// 为AppContext添加set_translation_handler方法
impl AppContext {
    pub fn set_translation_handler<F>(&mut self, handler: F)
    where
        F: Fn(&crate::bidirectional_translator::BidirectionalResult) + Send + Sync + 'static,
    {
        self.translator.lock().unwrap().set_result_callback(handler);
    }
}

/// 应用程序上下文，整合所有组件
pub struct AppContext {
    /// 双向翻译器
    translator: Arc<Mutex<BidirectionalTranslator>>,
    /// 虚拟音频管理器
    audio_manager: VirtualAudioManager,
}

impl AppContext {
    /// 创建新的应用程序上下文
    pub fn new(user_language: &str, other_language: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建双向翻译器
        let translator = Arc::new(Mutex::new(BidirectionalTranslator::new(user_language, other_language)?));
        
        // 创建虚拟音频管理器
        let mut audio_manager = VirtualAudioManager::new(Arc::clone(&translator))?;
        
        Ok(AppContext {
            translator,
            audio_manager,
        })
    }

    /// 初始化应用程序
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 设置翻译结果处理器
        let translator_clone = Arc::clone(&self.translator);
        self.audio_manager.set_translation_handler(move |result| {
            match result.direction {
                // 用户说话，翻译成对方语言 - 暂时不播放，仅用于记录
                crate::bidirectional_translator::TranslationDirection::UserToOther => {
                    println!("User spoke: '{}' -> Translated to '{}'", 
                             result.original_text, result.translated_text);
                }
                // 对方说话，翻译成用户语言 - 播放翻译后的音频
                crate::bidirectional_translator::TranslationDirection::OtherToUser => {
                    println!("Other spoke: '{}' -> Translated to '{}'", 
                             result.original_text, result.translated_text);
                    // 在实际实现中，这里会将翻译后的文本转为语音并播放
                }
            }
        });

        Ok(())
    }

    /// 启动应用程序
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 启动虚拟音频管理器
        self.audio_manager.start()?;

        // 启动双向翻译器
        self.translator.lock().unwrap().start()?;

        println!("Application started with virtual devices:");
        println!("  Input device: {}", self.audio_manager.get_virtual_input_device_id());
        println!("  Output device: {}", self.audio_manager.get_virtual_output_device_id());
        println!("  Language pair: {} <-> {}", 
                 self.translator.lock().unwrap().get_current_language_pair().source,
                 self.translator.lock().unwrap().get_current_language_pair().target);

        Ok(())
    }

    /// 停止应用程序
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 停止翻译器
        self.translator.lock().unwrap().stop()?;

        // 停止音频管理器
        self.audio_manager.stop()?;

        println!("Application stopped");
        Ok(())
    }

    /// 模拟用户说话
    pub async fn simulate_user_speaking(&self, audio_data: &[AudioSample]) {
        self.translator.lock().unwrap().simulate_user_speaking(audio_data).await;
    }

    /// 模拟对方说话
    pub fn simulate_other_speaking(&self, audio_data: &[AudioSample]) {
        self.audio_manager.simulate_receive_other_audio(audio_data);
    }

    /// 更新语言设置
    pub fn update_languages(&mut self, user_language: &str, other_language: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.translator.lock().unwrap().update_language_pair(user_language, other_language)
    }

    /// 获取当前语言对
    pub fn get_current_language_pair(&self) -> crate::bidirectional_translator::LanguagePair {
        self.translator.lock().unwrap().get_current_language_pair()
    }

    /// 切换到用户模式
    pub fn switch_to_user_mode(&self) {
        self.translator.lock().unwrap().switch_to_user_mode();
    }

    /// 更新语言对
    pub fn update_language_pair(&mut self, source_lang: &str, target_lang: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.translator.lock().unwrap().update_language_pair(source_lang, target_lang)
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> crate::bidirectional_translator::TranslationStats {
        self.translator.lock().unwrap().get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_audio_manager_creation() {
        let translator = Arc::new(Mutex::new(BidirectionalTranslator::new("zh", "en").unwrap()));
        let manager = VirtualAudioManager::new(translator);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_app_context_creation() {
        let mut app = AppContext::new("zh", "en").unwrap();
        assert!(app.initialize().is_ok());
        assert!(app.start().is_ok());
        assert!(app.stop().is_ok());
    }
}
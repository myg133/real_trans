//! 音频交换机模块
//! 实现全双工实时双向语音同传的音频路由

use std::sync::{Arc};
use std::collections::VecDeque;
use tokio::sync::{mpsc, Mutex as AsyncMutex};
use tokio::time::{sleep, Duration};
use crate::io::audio_device::{AudioDevice, AudioSample};
use crate::engine::translation_pipeline::TranslationPipeline;
use crate::bidirectional_translator::{BidirectionalTranslator};

/// 音频交换机状态
#[derive(Debug, Clone, PartialEq)]
pub enum AudioSwitchboardStatus {
    Idle,
    ProcessingOutbound,  // 处理发送端（物理麦克风→虚拟麦克风）
    ProcessingInbound,   // 处理接收端（系统环回→物理耳机）
    Correcting,          // 修正翻译结果
}

/// 音频交换机 - 管理四个音频流的路由
pub struct AudioSwitchboard {
    /// 发送端：物理麦克风 -> 虚拟麦克风
    outbound_pipeline: Option<Arc<AsyncMutex<TranslationPipeline>>>,
    /// 接收端：系统环回 -> 物理耳机
    inbound_pipeline: Option<Arc<AsyncMutex<TranslationPipeline>>>,
    /// 物理麦克风设备
    physical_mic: Option<Box<dyn AudioDevice>>,
    /// 物理耳机设备
    physical_headphones: Option<Box<dyn AudioDevice>>,
    /// 虚拟音频设备
    virtual_cable_input: Option<Box<dyn AudioDevice>>,  // 虚拟线缆输入端（会议软件的输入源）
    virtual_cable_output: Option<Box<dyn AudioDevice>>, // 虚拟线缆输出端（接收会议声音）
    /// 状态管理
    status: AudioSwitchboardStatus,
    /// 控制通道
    control_tx: Option<mpsc::UnboundedSender<AudioControl>>,
    /// 用于克隆接收器的Arc包装
    control_rx_clone: Arc<tokio::sync::Mutex<Option<mpsc::UnboundedReceiver<AudioControl>>>>,
    /// 音频缓冲区
    outbound_buffer: Arc<AsyncMutex<VecDeque<AudioSample>>>,
    inbound_buffer: Arc<AsyncMutex<VecDeque<AudioSample>>>,
    /// 翻译器实例
    translator: Arc<AsyncMutex<BidirectionalTranslator>>,
}

/// 音频控制消息
#[derive(Debug)]
pub enum AudioControl {
    Start,
    Stop,
    ToggleMute,
    SetLanguagePair(String, String),
    StatusRequest,
}

impl AudioSwitchboard {
    /// 创建新的音频交换机
    pub fn new(
        user_language: &str, 
        other_language: &str
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (control_tx, control_rx) = mpsc::unbounded_channel();
        
        // 创建双向翻译器
        let translator = Arc::new(AsyncMutex::new(BidirectionalTranslator::new(user_language, other_language)?));
        
        Ok(AudioSwitchboard {
            outbound_pipeline: None,
            inbound_pipeline: None,
            physical_mic: None,
            physical_headphones: None,
            virtual_cable_input: None,
            virtual_cable_output: None,
            status: AudioSwitchboardStatus::Idle,
            control_tx: Some(control_tx),
            control_rx_clone: Arc::new(tokio::sync::Mutex::new(Some(control_rx))),
            outbound_buffer: Arc::new(AsyncMutex::new(VecDeque::new())),
            inbound_buffer: Arc::new(AsyncMutex::new(VecDeque::new())),
            translator,
        })
    }

    /// 初始化音频设备
    pub fn initialize_devices(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 初始化物理麦克风
        // 注意：在实际实现中，这将连接到真实的物理麦克风
        self.physical_mic = Some(Box::new(crate::io::audio_device::MockAudioDevice::new()));
        
        // 初始化物理耳机
        self.physical_headphones = Some(Box::new(crate::io::audio_device::MockAudioDevice::new()));
        
        // 初始化虚拟音频设备（模拟VB-Cable或BlackHole）
        self.virtual_cable_input = Some(Box::new(crate::io::virtual_audio_device::VirtualAudioDevice::new(
            "virtual_mic_input", 
            "virtual_mic_output", 
            16000, 
            1
        )));
        
        self.virtual_cable_output = Some(Box::new(crate::io::virtual_audio_device::VirtualAudioDevice::new(
            "virtual_spk_input", 
            "virtual_spk_output", 
            16000, 
            1
        )));
        
        println!("Audio devices initialized");
        Ok(())
    }

    /// 启动音频交换机
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AudioSwitchboardStatus::Idle;
        
        // 初始化设备
        self.initialize_devices()?;
        
        // 启动发送端流水线（物理麦克风 -> 虚拟麦克风）
        self.setup_outbound_pipeline().await?;
        
        // 启动接收端流水线（系统环回 -> 物理耳机）
        self.setup_inbound_pipeline().await?;
        
        // 启动主控制循环
        self.start_control_loop().await;
        
        println!("Audio switchboard started with dual-channel workflow");
        Ok(())
    }

    /// 设置发送端流水线：物理麦克风 -> 虚拟麦克风
    async fn setup_outbound_pipeline(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut mic) = self.physical_mic {
            // 打开物理麦克风输入流
            mic.open_input_stream(Some("physical_mic".to_string()), Box::new({
                let translator = Arc::clone(&self.translator);
                
                move |audio_data| {
                    // 1. 将物理麦克风音频数据发送到翻译器（发送方向：用户语言 -> 对方语言）
                    let translator_clone = Arc::clone(&translator);
                    let audio_vec = audio_data.to_vec();
                    
                    // 在tokio运行时中执行异步操作
                    tokio::spawn(async move {
                        // 处理发送端：物理麦克风声音 -> 翻译 -> 虚拟麦克风输出
                        let _ = translator_clone.lock().await
                            .handle_outbound_audio(&audio_vec)  // 从用户到对方
                            .await;
                    });
                }
            }))?;
            
            mic.start_recording()?;
        }

        // 设置虚拟麦克风输出（会议软件将从此获取音频）
        if let Some(ref mut virtual_mic) = self.virtual_cable_input {
            virtual_mic.open_output_stream(Some("virtual_mic_output".to_string()))?;
        }

        println!("Outbound pipeline (Physical Mic -> Virtual Mic) configured");
        Ok(())
    }

    /// 设置接收端流水线：系统环回 -> 物理耳机
    async fn setup_inbound_pipeline(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 模拟系统环回音频捕获（实际实现中需要从系统音频输出捕获）
        if let Some(ref mut virtual_spk) = self.virtual_cable_output {
            virtual_spk.open_input_stream(Some("virtual_spk_input".to_string()), Box::new({
                let translator = Arc::clone(&self.translator);
                
                move |audio_data| {
                    // 1. 将系统环回音频数据发送到翻译器（接收方向：对方语言 -> 用户语言）
                    let translator_clone = Arc::clone(&translator);
                    let audio_vec = audio_data.to_vec();
                    
                    tokio::spawn(async move {
                        // 处理接收端：系统环回声音 -> 翻译 -> 物理耳机播放
                        let _ = translator_clone.lock().await
                            .handle_inbound_audio(&audio_vec)  // 从对方到用户
                            .await;
                    });
                }
            }))?;
            
            virtual_spk.start_recording()?;
        }

        // 设置物理耳机输出
        if let Some(ref mut headphones) = self.physical_headphones {
            headphones.open_output_stream(Some("physical_headphones".to_string()))?;
        }

        println!("Inbound pipeline (System Loopback -> Physical Headphones) configured");
        Ok(())
    }

    /// 启动控制循环
    async fn start_control_loop(&self) {
        let receiver_mutex = Arc::clone(&self.control_rx_clone);
        let translator = Arc::clone(&self.translator);
        
        tokio::spawn(async move {
            let mut receiver_opt = receiver_mutex.lock().await;
            let mut receiver = receiver_opt.take(); // 移动receiver的所有权
            drop(receiver_opt); // 释放锁
            
            if let Some(mut recv) = receiver {
                loop {
                    if let Some(control_msg) = recv.recv().await {
                        match control_msg {
                            AudioControl::Start => {
                                println!("Audio switchboard: Starting...");
                            },
                            AudioControl::Stop => {
                                println!("Audio switchboard: Stopping...");
                            },
                            AudioControl::SetLanguagePair(src, tgt) => {
                                println!("Audio switchboard: Changing language pair to {} -> {}", src, tgt);
                                let _ = translator.lock().await.update_language_pair(&src, &tgt);
                            },
                            AudioControl::ToggleMute => {
                                println!("Audio switchboard: Toggling mute");
                            },
                            AudioControl::StatusRequest => {
                                println!("Audio switchboard: Current status requested");
                            },
                        }
                    }
                }
            }
        });
    }

    /// 停止音频交换机
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.status = AudioSwitchboardStatus::Idle;
        
        // 停止所有设备
        if let Some(ref mut mic) = self.physical_mic {
            mic.stop_recording()?;
            mic.close_input_stream()?;
        }
        
        if let Some(ref mut headphones) = self.physical_headphones {
            headphones.close_output_stream()?;
        }
        
        if let Some(ref mut virtual_mic) = self.virtual_cable_input {
            virtual_mic.stop_recording()?;
            virtual_mic.close_input_stream()?;
            virtual_mic.close_output_stream()?;
        }
        
        if let Some(ref mut virtual_spk) = self.virtual_cable_output {
            virtual_spk.stop_recording()?;
            virtual_spk.close_input_stream()?;
        }

        println!("Audio switchboard stopped");
        Ok(())
    }

    /// 获取当前状态
    pub fn get_status(&self) -> AudioSwitchboardStatus {
        self.status.clone()
    }

    /// 发送控制消息
    pub fn send_control(&self, control: AudioControl) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref tx) = self.control_tx {
            tx.send(control)?;
        }
        Ok(())
    }

    /// 模拟物理麦克风输入（用于测试）
    pub async fn simulate_physical_mic_input(&self, audio_data: &[AudioSample]) {
        // 将音频数据传递给发送端翻译流水线
        let translator_clone = Arc::clone(&self.translator);
        let audio_vec = audio_data.to_vec();
        
        let _ = translator_clone.lock().await
            .handle_outbound_audio(&audio_vec)
            .await;
    }

    /// 模拟系统环回输入（用于测试）
    pub async fn simulate_system_loopback_input(&self, audio_data: &[AudioSample]) {
        // 将音频数据传递给接收端翻译流水线
        let translator_clone = Arc::clone(&self.translator);
        let audio_vec = audio_data.to_vec();
        
        let _ = translator_clone.lock().await
            .handle_inbound_audio(&audio_vec)
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_switchboard_creation() {
        let switchboard = AudioSwitchboard::new("zh", "en");
        assert!(switchboard.is_ok());
    }

    #[tokio::test]
    async fn test_audio_switchboard_lifecycle() {
        let mut switchboard = AudioSwitchboard::new("zh", "en").unwrap();
        
        // 测试初始化
        assert!(switchboard.initialize_devices().is_ok());
        
        // 测试启动
        assert!(switchboard.start().await.is_ok());
        
        // 测试状态
        assert_eq!(switchboard.get_status(), AudioSwitchboardStatus::Idle);
        
        // 测试停止
        assert!(switchboard.stop().is_ok());
    }
}
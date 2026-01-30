//! 音频设备抽象接口
//! 提供跨平台音频设备的统一接口

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

/// 音频设备信息结构
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub id: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub is_default: bool,
}

/// 音频采样类型
pub type AudioSample = i16;

/// 音频设备抽象 trait
pub trait AudioDevice: Send + Sync {
    /// 获取所有可用的音频输入设备
    fn get_available_input_devices(&self) -> Vec<DeviceInfo>;
    
    /// 获取所有可用的音频输出设备
    fn get_available_output_devices(&self) -> Vec<DeviceInfo>;
    
    /// 获取默认音频输入设备
    fn get_default_input_device(&self) -> DeviceInfo;
    
    /// 获取默认音频输出设备
    fn get_default_output_device(&self) -> DeviceInfo;
    
    /// 打开音频输入流
    fn open_input_stream(
        &mut self,
        device_id: Option<String>,
        callback: Box<dyn Fn(&[AudioSample]) + Send>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 关闭音频输入流
    fn close_input_stream(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 打开音频输出流
    fn open_output_stream(
        &mut self,
        device_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 关闭音频输出流
    fn close_output_stream(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 开始录音
    fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 停止录音
    fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// 播放音频数据
    fn play_audio(&mut self, data: &[AudioSample]) -> Result<usize, Box<dyn std::error::Error>>;
    
    /// 检查输入流是否正在运行
    fn is_recording(&self) -> bool;
}

/// 模拟音频设备实现
pub struct MockAudioDevice {
    is_recording: bool,
    input_device_id: String,
    output_device_id: String,
}

impl MockAudioDevice {
    pub fn new() -> Self {
        MockAudioDevice {
            is_recording: false,
            input_device_id: "mock_input".to_string(),
            output_device_id: "mock_output".to_string(),
        }
    }
}

impl AudioDevice for MockAudioDevice {
    fn get_available_input_devices(&self) -> Vec<DeviceInfo> {
        vec![DeviceInfo {
            name: "Mock Input Device".to_string(),
            id: "mock_input".to_string(),
            sample_rate: super::super::SAMPLE_RATE,
            channels: super::super::CHANNELS,
            is_default: true,
        }]
    }

    fn get_available_output_devices(&self) -> Vec<DeviceInfo> {
        vec![DeviceInfo {
            name: "Mock Output Device".to_string(),
            id: "mock_output".to_string(),
            sample_rate: super::super::SAMPLE_RATE,
            channels: super::super::CHANNELS,
            is_default: true,
        }]
    }

    fn get_default_input_device(&self) -> DeviceInfo {
        self.get_available_input_devices()[0].clone()
    }

    fn get_default_output_device(&self) -> DeviceInfo {
        self.get_available_output_devices()[0].clone()
    }

    fn open_input_stream(
        &mut self,
        device_id: Option<String>,
        _callback: Box<dyn Fn(&[AudioSample]) + Send>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.input_device_id = device_id.unwrap_or_else(|| "mock_input".to_string());
        println!("Opened input stream on device: {}", self.input_device_id);
        Ok(())
    }

    fn close_input_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Closed input stream on device: {}", self.input_device_id);
        Ok(())
    }

    fn open_output_stream(
        &mut self,
        device_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.output_device_id = device_id.unwrap_or_else(|| "mock_output".to_string());
        println!("Opened output stream on device: {}", self.output_device_id);
        Ok(())
    }

    fn close_output_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Closed output stream on device: {}", self.output_device_id);
        Ok(())
    }

    fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_recording = true;
        println!("Started recording");
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_recording = false;
        println!("Stopped recording");
        Ok(())
    }

    fn play_audio(&mut self, data: &[AudioSample]) -> Result<usize, Box<dyn std::error::Error>> {
        if data.is_empty() {
            return Ok(0);
        }
        
        println!("Playing {} samples of audio", data.len());
        Ok(data.len())
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}
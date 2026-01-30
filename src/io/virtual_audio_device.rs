//! 虚拟音频设备实现
//! 提供在测试环境中模拟真实音频设备行为的能力

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tokio::time::{sleep, Duration};
use crate::io::audio_device::{AudioDevice, DeviceInfo, AudioSample};

/// 虚拟音频设备实现
/// 模拟真实的音频设备行为，包括数据缓冲和异步处理
pub struct VirtualAudioDevice {
    is_recording: bool,
    input_device_id: String,
    output_device_id: String,
    sample_rate: u32,
    channels: u16,
    /// 输入音频缓冲区
    input_buffer: Arc<Mutex<VecDeque<AudioSample>>>,
    /// 输出音频缓冲区
    output_buffer: Arc<Mutex<VecDeque<AudioSample>>>,
    /// 输入回调函数
    input_callback: Option<Box<dyn Fn(&[AudioSample]) + Send + Sync>>,
}

impl VirtualAudioDevice {
    /// 创建新的虚拟音频设备
    pub fn new(input_device_id: &str, output_device_id: &str, sample_rate: u32, channels: u16) -> Self {
        VirtualAudioDevice {
            is_recording: false,
            input_device_id: input_device_id.to_string(),
            output_device_id: output_device_id.to_string(),
            sample_rate,
            channels,
            input_buffer: Arc::new(Mutex::new(VecDeque::new())),
            output_buffer: Arc::new(Mutex::new(VecDeque::new())),
            input_callback: None,
        }
    }

    /// 模拟从虚拟输入设备读取音频数据
    pub fn simulate_input_data(&self, data: &[AudioSample]) {
        if let Some(ref callback) = self.input_callback {
            callback(data);
        }
    }

    /// 获取输出缓冲区中的数据
    pub fn get_output_data(&self) -> Vec<AudioSample> {
        let mut buffer = self.output_buffer.lock().unwrap();
        let result: Vec<AudioSample> = buffer.drain(..).collect();
        result
    }

    /// 模拟将数据写入虚拟输出设备
    fn write_to_output(&self, data: &[AudioSample]) {
        let mut buffer = self.output_buffer.lock().unwrap();
        for &sample in data {
            buffer.push_back(sample);
        }
    }
}

impl AudioDevice for VirtualAudioDevice {
    fn get_available_input_devices(&self) -> Vec<DeviceInfo> {
        vec![DeviceInfo {
            name: format!("Virtual Input Device ({})", self.input_device_id),
            id: self.input_device_id.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
            is_default: true,
        }]
    }

    fn get_available_output_devices(&self) -> Vec<DeviceInfo> {
        vec![DeviceInfo {
            name: format!("Virtual Output Device ({})", self.output_device_id),
            id: self.output_device_id.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
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
        callback: Box<dyn Fn(&[AudioSample]) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.input_device_id = device_id.unwrap_or_else(|| self.input_device_id.clone());
        self.input_callback = Some(callback);
        println!("Opened input stream on virtual device: {}", self.input_device_id);
        Ok(())
    }

    fn close_input_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Closed input stream on virtual device: {}", self.input_device_id);
        Ok(())
    }

    fn open_output_stream(
        &mut self,
        device_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.output_device_id = device_id.unwrap_or_else(|| self.output_device_id.clone());
        println!("Opened output stream on virtual device: {}", self.output_device_id);
        Ok(())
    }

    fn close_output_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Closed output stream on virtual device: {}", self.output_device_id);
        Ok(())
    }

    fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_recording = true;
        println!("Started recording on virtual device: {}", self.input_device_id);
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_recording = false;
        println!("Stopped recording on virtual device: {}", self.input_device_id);
        Ok(())
    }

    fn play_audio(&mut self, data: &[AudioSample]) -> Result<usize, Box<dyn std::error::Error>> {
        if data.is_empty() {
            return Ok(0);
        }

        // 模拟将音频数据发送到虚拟输出设备
        self.write_to_output(data);
        println!("Sent {} samples to virtual output device: {}", 
                 data.len(), self.output_device_id);
        Ok(data.len())
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_audio_device_creation() {
        let device = VirtualAudioDevice::new("virtual_input", "virtual_output", 44100, 1);
        assert_eq!(device.get_default_input_device().id, "virtual_input");
        assert_eq!(device.get_default_output_device().id, "virtual_output");
    }

    #[test]
    fn test_virtual_audio_device_callbacks() {
        let device = VirtualAudioDevice::new("virtual_input", "virtual_output", 44100, 1);
        let device_arc = Arc::new(Mutex::new(device));
        
        // 测试输入回调
        let callback_called = Arc::new(Mutex::new(false));
        let callback_flag = Arc::clone(&callback_called);
        
        // 注意：这个测试展示了我们需要如何处理回调
    }
}
//! 音频捕获模块
//! 管理音频采集流程，包括启动/停止录音、处理音频数据等

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

use super::audio_device::{AudioDevice, MockAudioDevice};
use crate::{AudioSample, SAMPLES_PER_FRAME};

/// 音频数据回调函数类型
pub type DataCallback = Box<dyn Fn(&[AudioSample], bool) + Send>;

/// 音频捕获结构体
pub struct AudioCapture {
    audio_device: Option<Box<dyn AudioDevice>>,
    data_callback: Option<DataCallback>,
    initialized: AtomicBool,
    capturing: AtomicBool,
    total_samples: std::sync::atomic::AtomicU64,
    sample_buffer: Arc<Mutex<Vec<AudioSample>>>,
}

impl AudioCapture {
    /// 创建新的音频捕获实例
    pub fn new() -> Self {
        AudioCapture {
            audio_device: None,
            data_callback: None,
            initialized: AtomicBool::new(false),
            capturing: AtomicBool::new(false),
            total_samples: std::sync::atomic::AtomicU64::new(0),
            sample_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 设置音频数据回调函数
    pub fn set_data_callback(&mut self, callback: DataCallback) {
        self.data_callback = Some(callback);
    }

    /// 初始化音频捕获
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized.load(Ordering::SeqCst) {
            return Ok(());
        }

        // 创建模拟音频设备
        let mut device = Box::new(MockAudioDevice::new());

        // 打开音频输入流
        device.open_input_stream(None, Box::new(|_data| {
            // 这里应该是实际的音频回调处理
        }))?;

        self.audio_device = Some(device);
        self.initialized.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// 启动音频捕获
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.initialized.load(Ordering::SeqCst) {
            self.initialize()?;
        }

        if self.capturing.load(Ordering::SeqCst) {
            return Ok(()); // 已经在捕获
        }

        if let Some(ref mut device) = self.audio_device {
            device.start_recording()?;
        }

        self.capturing.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// 停止音频捕获
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.capturing.load(Ordering::SeqCst) {
            return Ok(()); // 已经停止
        }

        if let Some(ref mut device) = self.audio_device {
            device.stop_recording()?;
        }

        self.capturing.store(false, Ordering::SeqCst);

        Ok(())
    }

    /// 检查是否正在捕获音频
    pub fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    /// 获取已捕获的总采样数
    pub fn get_total_samples(&self) -> u64 {
        self.total_samples.load(Ordering::SeqCst)
    }

    /// 模拟处理音频数据的方法
    pub async fn simulate_audio_input(&self, audio_data: &[AudioSample]) {
        if !self.capturing.load(Ordering::SeqCst) {
            return;
        }

        // 更新总采样数
        self.total_samples
            .fetch_add(audio_data.len() as u64, Ordering::SeqCst);

        // 调用回调函数传递音频数据
        if let Some(ref callback) = self.data_callback {
            callback(audio_data, false); // is_final = false
        }
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        if self.capturing.load(Ordering::SeqCst) {
            let _ = self.stop();
        }
    }
}
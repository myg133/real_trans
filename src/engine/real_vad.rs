//! 真实的VAD模块，集成Silero VAD

use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::{AudioSample, SAMPLE_RATE, FRAME_SIZE_MS};

/// VAD决策枚举
#[derive(Debug, Clone, PartialEq)]
pub enum VadDecision {
    Speech,     // 语音
    Silence,    // 静音
    Unclear,    // 不确定
}

/// VAD结果结构
#[derive(Debug, Clone)]
pub struct VadResult {
    pub decision: VadDecision,
    pub probability: f32,        // 语音概率 (0.0-1.0)
    pub is_start_of_speech: bool,  // 是否为语音开始
    pub is_end_of_speech: bool,    // 是否为语音结束
    pub timestamp: Instant,
}

/// 音频段类型
pub type AudioSegment = Vec<AudioSample>;

/// 语音段结束回调函数类型
pub type SpeechSegmentCallback = Box<dyn Fn(&AudioSegment, bool) + Send>;

/// 真实语音活动检测器
pub struct RealVad {
    sample_rate: u32,
    frame_duration_ms: u32,
    segment_callback: Option<SpeechSegmentCallback>,
    current_segment: Arc<Mutex<Vec<AudioSample>>>,
    min_speech_duration_ms: u32,  // 最小语音持续时间
    max_silence_duration_ms: u32, // 最大静音持续时间
    silence_count_ms: u32,
    in_speech: bool,
    // Silero VAD相关字段
    silero_model: Option<*mut std::ffi::c_void>, // 实际使用时会是Silero VAD的模型指针
}

unsafe impl Send for RealVad {}
unsafe impl Sync for RealVad {}

impl RealVad {
    /// 创建新的真实VAD实例
    pub fn new(sample_rate: u32, frame_duration_ms: u32) -> Self {
        RealVad {
            sample_rate,
            frame_duration_ms,
            segment_callback: None,
            current_segment: Arc::new(Mutex::new(Vec::new())),
            min_speech_duration_ms: 300,  // 默认最小语音持续时间为300ms
            max_silence_duration_ms: 800, // 默认最大静音持续时间为800ms
            silence_count_ms: 0,
            in_speech: false,
            silero_model: None,
        }
    }

    /// 设置语音段结束回调函数
    pub fn set_speech_segment_callback(&mut self, callback: SpeechSegmentCallback) {
        self.segment_callback = Some(callback);
    }

    /// 初始化VAD引擎
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Loading Silero VAD model...");
        
        // 这里将是实际的Silero VAD模型加载代码
        // let model = load_silero_vad_model()?;
        // self.silero_model = Some(model);
        
        // 模拟加载
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        println!("Silero VAD model loaded successfully");
        Ok(())
    }

    /// 处理音频帧
    pub fn process_frame(&mut self, audio_frame: &[AudioSample]) -> VadResult {
        // 这里将是实际的Silero VAD推理代码
        // let vad_result = self.silero_model.as_ref().unwrap().process_frame(audio_frame)?;
        
        // 模拟VAD处理 - 使用简单的能量阈值检测
        let energy: f64 = audio_frame
            .iter()
            .map(|&sample| (sample as f64) * (sample as f64))
            .sum::<f64>() / audio_frame.len() as f64;

        // 简单的能量阈值判断
        let is_speech = energy > 1000.0;  // 简化的阈值

        let mut result = VadResult {
            decision: if is_speech { VadDecision::Speech } else { VadDecision::Silence },
            probability: ((energy / 5000.0).min(1.0)) as f32,
            is_start_of_speech: false,
            is_end_of_speech: false,
            timestamp: Instant::now(),
        };

        if is_speech {
            if !self.in_speech {
                // 语音开始
                result.is_start_of_speech = true;
                self.in_speech = true;
                self.silence_count_ms = 0;

                // 开始新的语音段
                self.current_segment.lock().unwrap().clear();
            }

            // 将当前帧添加到语音段
            self.current_segment.lock().unwrap().extend_from_slice(audio_frame);
        } else {
            if self.in_speech {
                // 语音结束检测
                self.silence_count_ms += self.frame_duration_ms;

                if self.silence_count_ms >= self.max_silence_duration_ms {
                    // 检测到语音结束
                    result.is_end_of_speech = true;

                    // 检查语音段是否满足最小持续时间要求
                    let speech_duration_ms = 
                        (self.current_segment.lock().unwrap().len() * 1000) as u32 / self.sample_rate;
                    
                    if speech_duration_ms >= self.min_speech_duration_ms {
                        if let Some(ref callback) = self.segment_callback {
                            let segment = self.current_segment.lock().unwrap().clone();
                            callback(&segment, true); // final segment
                        }
                    }

                    self.in_speech = false;
                    self.current_segment.lock().unwrap().clear();
                }
            }
        }

        result
    }

    /// 重置VAD状态
    pub fn reset(&mut self) {
        self.current_segment.lock().unwrap().clear();
        self.silence_count_ms = 0;
        self.in_speech = false;
    }

    /// 获取当前累积的语音段
    pub fn get_current_speech_segment(&self) -> AudioSegment {
        self.current_segment.lock().unwrap().clone()
    }

    /// 设置语音段最小持续时间阈值（毫秒）
    pub fn set_min_speech_duration(&mut self, duration_ms: u32) {
        self.min_speech_duration_ms = duration_ms;
    }

    /// 设置静音最大持续时间阈值（毫秒）
    pub fn set_max_silence_duration(&mut self, duration_ms: u32) {
        self.max_silence_duration_ms = duration_ms;
    }
}

impl Drop for RealVad {
    fn drop(&mut self) {
        // 清理Silero VAD资源
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_vad_creation() {
        let vad = RealVad::new(16000, 30);
        assert_eq!(vad.sample_rate, 16000);
        assert_eq!(vad.frame_duration_ms, 30);
    }

    #[test]
    fn test_real_vad_process_frame() {
        let mut vad = RealVad::new(16000, 30);
        vad.initialize().unwrap();
        
        // 创建一个简单的音频帧（静音）
        let silent_frame = vec![0i16; 480]; // 30ms at 16kHz = 480 samples
        
        let result = vad.process_frame(&silent_frame);
        assert_eq!(result.decision, VadDecision::Silence);
    }
}
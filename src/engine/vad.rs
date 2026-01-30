//! 语音活动检测（Voice Activity Detection）模块
//! 用于检测音频流中的语音活动，过滤静音段，减少不必要的推理

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

/// 语音活动检测器
pub struct Vad {
    sample_rate: u32,
    frame_duration_ms: u32,
    segment_callback: Option<SpeechSegmentCallback>,
    current_segment: Arc<Mutex<Vec<AudioSample>>>,
    min_speech_duration_ms: u32,  // 最小语音持续时间
    max_silence_duration_ms: u32, // 最大静音持续时间
    silence_count_ms: u32,
    in_speech: bool,
    energy_threshold: f64,  // 能量阈值
}

impl Vad {
    /// 创建新的VAD实例
    pub fn new(sample_rate: u32, frame_duration_ms: u32) -> Self {
        Vad {
            sample_rate,
            frame_duration_ms,
            segment_callback: None,
            current_segment: Arc::new(Mutex::new(Vec::new())),
            min_speech_duration_ms: 300,  // 默认最小语音持续时间为300ms
            max_silence_duration_ms: 800, // 默认最大静音持续时间为800ms
            silence_count_ms: 0,
            in_speech: false,
            energy_threshold: 1000.0,  // 默认能量阈值
        }
    }

    /// 设置语音段结束回调函数
    pub fn set_speech_segment_callback(&mut self, callback: SpeechSegmentCallback) {
        self.segment_callback = Some(callback);
    }

    /// 处理音频帧
    pub fn process_frame(&mut self, audio_frame: &[AudioSample]) -> VadResult {
        // 计算音频能量
        let energy: f64 = audio_frame
            .iter()
            .map(|&sample| (sample as f64) * (sample as f64))
            .sum::<f64>() / audio_frame.len() as f64;

        // 简单的能量阈值判断
        let is_speech = energy > self.energy_threshold;

        let mut result = VadResult {
            decision: if is_speech { VadDecision::Speech } else { VadDecision::Silence },
            probability: ((energy / (self.energy_threshold * 5.0)).min(1.0)) as f32,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_creation() {
        let vad = Vad::new(16000, 30);
        assert_eq!(vad.sample_rate, 16000);
        assert_eq!(vad.frame_duration_ms, 30);
    }

    #[test]
    fn test_vad_process_frame() {
        let mut vad = Vad::new(16000, 30);
        
        // 创建一个简单的音频帧（静音）
        let silent_frame = vec![0i16; 480]; // 30ms at 16kHz = 480 samples
        
        let result = vad.process_frame(&silent_frame);
        assert_eq!(result.decision, VadDecision::Silence);
    }
}
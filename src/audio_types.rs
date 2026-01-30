/// 音频相关的常量和类型定义

// 音频采样参数
pub const SAMPLE_RATE: u32 = 16000;        // 采样率：16kHz
pub const CHANNELS: u16 = 1;               // 声道数：单声道
pub const BITS_PER_SAMPLE: u16 = 16;       // 位深度：16位
pub const FRAME_SIZE_MS: u32 = 20;         // 帧大小：20ms
pub const SAMPLES_PER_FRAME: usize = (SAMPLE_RATE as usize * FRAME_SIZE_MS as usize) / 1000;  // 每帧样本数

// 音频数据类型别名
pub type AudioSample = i16;                // 音频采样点类型

// 音频缓冲区大小
pub const DEFAULT_RING_BUFFER_SIZE: usize = 8192;   // 默认环形缓冲区大小

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub samples: [AudioSample; SAMPLES_PER_FRAME],
    pub timestamp: std::time::Instant,
}

impl AudioFrame {
    pub fn new() -> Self {
        AudioFrame {
            samples: [0; SAMPLES_PER_FRAME],
            timestamp: std::time::Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_constants() {
        assert_eq!(SAMPLE_RATE, 16000);
        assert_eq!(CHANNELS, 1);
        assert_eq!(BITS_PER_SAMPLE, 16);
        assert_eq!(FRAME_SIZE_MS, 20);
        assert_eq!(SAMPLES_PER_FRAME, 320); // 16000 * 20 / 1000 = 320
    }
}
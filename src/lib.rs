//! 实时同声翻译应用（Real-time Speech Translation, RST）
//! 一个专注于本地化、低延时、跨平台的实时翻译系统

pub mod audio_types;
pub mod core {
    pub mod ring_buffer;
}

pub use core::ring_buffer::RingBuffer;
pub use audio_types::*;

/// IO模块 - 负责音频输入输出
pub mod io {
    pub mod audio_device;
    pub mod audio_capture;
}

/// Engine模块 - 负责ASR、MT、TTS核心引擎
pub mod engine {
    pub mod vad;
    pub mod asr;
    pub mod mt;
    pub mod tts;
    pub mod translation_pipeline;
}
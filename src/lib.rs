//! 实时同声翻译应用（Real-time Speech Translation, RST）
//! 一个专注于本地化、低延时、跨平台的实时翻译系统

pub mod audio_types;
pub mod core {
    pub mod ring_buffer;
}

pub use core::ring_buffer::RingBuffer;
pub use audio_types::*;
pub use io::physical_device::PhysicalAudioDevice;

/// IO模块 - 负责音频输入输出
pub mod io {
    pub mod audio_device;
    pub mod virtual_audio_device;
    pub mod audio_capture;
    pub mod physical_device;
}

/// Engine模块 - 负责ASR、MT、TTS核心引擎
pub mod engine {
    pub mod vad;
    pub mod asr;
    pub mod mt;
    pub mod tts;
    pub mod translation_pipeline;
}

/// 双向翻译器 - 实现用户语言和对方语言之间的双向实时翻译
pub mod bidirectional_translator;

/// 虚拟音频设备管理器 - 管理虚拟音频输入和输出设备
pub mod virtual_audio_manager;

/// 音频交换机 - 实现全双工实时双向语音同传的音频路由
pub mod audio_switchboard;

/// 测试模块 - 包含各种测试工具
pub mod tests;
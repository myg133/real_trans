//! 物理音频设备实现
//! 使用cpal库连接到真实的音频硬件设备

use std::sync::{Arc, Mutex};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Sample, Stream, StreamConfig,
};
use crate::{
    audio_types::{AudioSample, SAMPLE_RATE, CHANNELS},
    io::audio_device::{AudioDevice, DeviceInfo},
};

/// 物理音频设备实现
pub struct PhysicalAudioDevice {
    host: cpal::Host,
    input_device: Option<Device>,
    output_device: Option<Device>,
    is_recording: Arc<Mutex<bool>>,
    recording_stream: Arc<Mutex<Option<Stream>>>,
    playback_stream: Arc<Mutex<Option<Stream>>>,
}

impl PhysicalAudioDevice {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        
        // 获取默认输入和输出设备
        let input_device = host.default_input_device()
            .ok_or("No input device available")?;
        let output_device = host.default_output_device()
            .ok_or("No output device available")?;

        println!("Default input device: {:?}", input_device.name()?);
        println!("Default output device: {:?}", output_device.name()?);

        Ok(PhysicalAudioDevice {
            host,
            input_device: Some(input_device),
            output_device: Some(output_device),
            is_recording: Arc::new(Mutex::new(false)),
            recording_stream: Arc::new(Mutex::new(None)),
            playback_stream: Arc::new(Mutex::new(None)),
        })
    }

    /// 列出所有可用的输入设备
    pub fn list_input_devices() -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        
        if let Ok(default_device) = host.default_input_device().ok_or("No default input device") {
            let default_name = default_device.name()?;
            devices.push(DeviceInfo {
                name: format!("{} (default)", default_name),
                id: default_device.name().unwrap_or_else(|_| "unknown".to_string()),
                sample_rate: SAMPLE_RATE,
                channels: CHANNELS,
                is_default: true,
            });
        }
        
        for device in host.input_devices()? {
            let name = device.name()?;
            // 避免重复添加默认设备
            if !devices.iter().any(|d| d.id == name) {
                devices.push(DeviceInfo {
                    name: name.clone(),
                    id: name,
                    sample_rate: SAMPLE_RATE,
                    channels: CHANNELS,
                    is_default: false,
                });
            }
        }
        
        Ok(devices)
    }

    /// 列出所有可用的输出设备
    pub fn list_output_devices() -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        
        if let Ok(default_device) = host.default_output_device().ok_or("No default output device") {
            let default_name = default_device.name()?;
            devices.push(DeviceInfo {
                name: format!("{} (default)", default_name),
                id: default_device.name().unwrap_or_else(|_| "unknown".to_string()),
                sample_rate: SAMPLE_RATE,
                channels: CHANNELS,
                is_default: true,
            });
        }
        
        for device in host.output_devices()? {
            let name = device.name()?;
            // 避免重复添加默认设备
            if !devices.iter().any(|d| d.id == name) {
                devices.push(DeviceInfo {
                    name: name.clone(),
                    id: name,
                    sample_rate: SAMPLE_RATE,
                    channels: CHANNELS,
                    is_default: false,
                });
            }
        }
        
        Ok(devices)
    }

    /// 根据名称选择输入设备
    pub fn select_input_device_by_name(&mut self, device_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let device = Self::list_input_devices()?
            .iter()
            .find(|device| device.name.contains(device_name) || device.id.contains(device_name))
            .and_then(|device_info| {
                self.host.input_devices().ok()
                    .and_then(|mut devices| {
                        devices.find(|device| {
                            device.name().map(|name| name.contains(&device_info.id)).unwrap_or(false)
                        })
                    })
            });

        match device {
            Some(device) => {
                self.input_device = Some(device);
                println!("Selected input device: {}", device_name);
                Ok(())
            }
            None => Err(format!("Input device '{}' not found", device_name).into()),
        }
    }

    /// 根据名称选择输出设备
    pub fn select_output_device_by_name(&mut self, device_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let device = Self::list_output_devices()?
            .iter()
            .find(|device| device.name.contains(device_name) || device.id.contains(device_name))
            .and_then(|device_info| {
                self.host.output_devices().ok()
                    .and_then(|mut devices| {
                        devices.find(|device| {
                            device.name().map(|name| name.contains(&device_info.id)).unwrap_or(false)
                        })
                    })
            });

        match device {
            Some(device) => {
                self.output_device = Some(device);
                println!("Selected output device: {}", device_name);
                Ok(())
            }
            None => Err(format!("Output device '{}' not found", device_name).into()),
        }
    }
}

impl AudioDevice for PhysicalAudioDevice {
    fn get_available_input_devices(&self) -> Vec<DeviceInfo> {
        Self::list_input_devices().unwrap_or_default()
    }

    fn get_available_output_devices(&self) -> Vec<DeviceInfo> {
        Self::list_output_devices().unwrap_or_default()
    }

    fn get_default_input_device(&self) -> DeviceInfo {
        if let Some(ref device) = self.input_device {
            DeviceInfo {
                name: device.name().unwrap_or_else(|_| "Unknown Input Device".to_string()),
                id: device.name().unwrap_or_else(|_| "unknown_input".to_string()),
                sample_rate: SAMPLE_RATE,
                channels: CHANNELS,
                is_default: true,
            }
        } else {
            DeviceInfo {
                name: "No Input Device".to_string(),
                id: "no_input".to_string(),
                sample_rate: SAMPLE_RATE,
                channels: CHANNELS,
                is_default: false,
            }
        }
    }

    fn get_default_output_device(&self) -> DeviceInfo {
        if let Some(ref device) = self.output_device {
            DeviceInfo {
                name: device.name().unwrap_or_else(|_| "Unknown Output Device".to_string()),
                id: device.name().unwrap_or_else(|_| "unknown_output".to_string()),
                sample_rate: SAMPLE_RATE,
                channels: CHANNELS,
                is_default: true,
            }
        } else {
            DeviceInfo {
                name: "No Output Device".to_string(),
                id: "no_output".to_string(),
                sample_rate: SAMPLE_RATE,
                channels: CHANNELS,
                is_default: false,
            }
        }
    }

    fn open_input_stream(
        &mut self,
        device_id: Option<String>,
        callback: Box<dyn Fn(&[AudioSample]) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let device = if let Some(id) = &device_id {
            // 尝试根据ID查找设备
            self.host.input_devices()?
                .find(|device| {
                    device.name().map(|name| name.contains(id)).unwrap_or(false)
                })
                .ok_or_else(|| format!("Input device '{}' not found", id))?
        } else {
            self.input_device.clone().ok_or("No input device selected")?
        };

        // 获取设备支持的配置
        let config = device.default_input_config()
            .map_err(|_| "Failed to get default input config")?;

        let sample_format = config.sample_format();
        let config: StreamConfig = config.into();

        let err_fn = |err| eprintln!("An error occurred on the input audio stream: {}", err);
        
        // 创建回调函数的线程安全包装
        let callback = Arc::new(callback);
        
        // 根据采样格式创建流
        let stream = match sample_format {
            cpal::SampleFormat::I16 => {
                build_input_stream_typed::<i16>(&device, &config, Arc::clone(&callback), err_fn)?
            }
            cpal::SampleFormat::U16 => {
                build_input_stream_typed::<u16>(&device, &config, Arc::clone(&callback), err_fn)?
            }
            cpal::SampleFormat::F32 => {
                build_input_stream_typed::<f32>(&device, &config, Arc::clone(&callback), err_fn)?
            }
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format).into()),
        };

        // 将流存储到共享变量中
        *self.recording_stream.lock().unwrap() = Some(stream);
        println!("Opened input stream on device: {}", device.name()?);
        Ok(())
    }

    fn close_input_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        *self.recording_stream.lock().unwrap() = None;
        println!("Closed input stream");
        Ok(())
    }

    fn open_output_stream(
        &mut self,
        device_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let device = if let Some(id) = &device_id {
            // 尝试根据ID查找设备
            self.host.output_devices()?
                .find(|device| {
                    device.name().map(|name| name.contains(id)).unwrap_or(false)
                })
                .ok_or_else(|| format!("Output device '{}' not found", id))?
        } else {
            self.output_device.clone().ok_or("No output device selected")?
        };

        // 获取设备支持的配置
        let config = device.default_output_config()
            .map_err(|_| "Failed to get default output config")?;

        let sample_format = config.sample_format();
        let config: StreamConfig = config.into();

        let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);
        
        // 根据采样格式创建流
        let stream = match sample_format {
            cpal::SampleFormat::I16 => {
                build_output_stream_typed::<i16>(&device, &config, err_fn)?
            }
            cpal::SampleFormat::U16 => {
                build_output_stream_typed::<u16>(&device, &config, err_fn)?
            }
            cpal::SampleFormat::F32 => {
                build_output_stream_typed::<f32>(&device, &config, err_fn)?
            }
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format).into()),
        };

        // 将流存储到共享变量中
        *self.playback_stream.lock().unwrap() = Some(stream);
        println!("Opened output stream on device: {}", device.name()?);
        Ok(())
    }

    fn close_output_stream(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        *self.playback_stream.lock().unwrap() = None;
        println!("Closed output stream");
        Ok(())
    }

    fn start_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream_guard = self.recording_stream.lock().unwrap();
        if let Some(ref stream) = *stream_guard {
            stream.play()?;
            *self.is_recording.lock().unwrap() = true;
            println!("Started recording");
        } else {
            return Err("No input stream opened".into());
        }
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream_guard = self.recording_stream.lock().unwrap();
        if let Some(ref stream) = *stream_guard {
            stream.pause()?;
            *self.is_recording.lock().unwrap() = false;
            println!("Stopped recording");
        } else {
            return Err("No input stream opened".into());
        }
        Ok(())
    }

    fn play_audio(&mut self, data: &[AudioSample]) -> Result<usize, Box<dyn std::error::Error>> {
        // 注意：对于实时播放，我们需要预先建立输出流并在回调中提供数据
        // 这里只是模拟播放行为
        if data.is_empty() {
            return Ok(0);
        }
        
        println!("Queued {} samples for playback", data.len());
        Ok(data.len())
    }

    fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }
}

// 辅助函数：为特定类型构建输入流
fn build_input_stream_typed<T>(
    device: &Device,
    config: &StreamConfig,
    callback: Arc<dyn Fn(&[AudioSample]) + Send + Sync>,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<Stream, Box<dyn std::error::Error>>
where
    T: Sample + Into<AudioSample> + Send + 'static,
{
    let channels = config.channels as usize;
    
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            // 将采样数据转换为AudioSample类型
            let samples: Vec<AudioSample> = data.iter()
                .map(|&sample| sample.into())
                .collect();
            
            callback(&samples);
        },
        err_fn,
        None,
    )?;
    
    Ok(stream)
}

// 辅助函数：为特定类型构建输出流
fn build_output_stream_typed<T>(
    device: &Device,
    config: &StreamConfig,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<Stream, Box<dyn std::error::Error>>
where
    T: Sample + From<AudioSample> + Send + 'static,
{
    // 创建一个共享缓冲区用于输出
    let output_buffer = Arc::new(Mutex::new(Vec::new()));
    let buffer_clone = output_buffer.clone();
    
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // 这里应该从外部获取要播放的音频数据
            // 当前只是填充静音数据
            for frame in data.chunks_mut(config.channels as usize) {
                for sample_out in frame.iter_mut() {
                    *sample_out = (0i16).into();
                }
            }
        },
        err_fn,
        None,
    )?;
    
    Ok(stream)
}
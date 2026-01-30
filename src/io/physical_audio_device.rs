//! 物理音频设备实现
//! 连接到真实的音频硬件设备

use std::sync::{Arc, Mutex};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, SupportedStreamConfig,
};
use crate::audio_types::{AudioSample, SAMPLE_RATE, CHANNELS};

pub struct PhysicalAudioDevice {
    input_device: Option<Device>,
    output_device: Option<Device>,
    sample_rate: u32,
    channels: u16,
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
            input_device: Some(input_device),
            output_device: Some(output_device),
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
        })
    }

    pub fn list_input_devices() -> Result<Vec<(String, Device)>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let devices = host.input_devices()?
            .map(|device| {
                let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
                (name, device)
            })
            .collect();
        
        Ok(devices)
    }

    pub fn list_output_devices() -> Result<Vec<(String, Device)>, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let devices = host.output_devices()?
            .map(|device| {
                let name = device.name().unwrap_or_else(|_| "Unknown Device".to_string());
                (name, device)
            })
            .collect();
        
        Ok(devices)
    }

    pub fn select_input_device_by_name(&mut self, device_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.input_devices()?
            .find(|device| {
                if let Ok(name) = device.name() {
                    name.contains(device_name)
                } else {
                    false
                }
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

    pub fn select_output_device_by_name(&mut self, device_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.output_devices()?
            .find(|device| {
                if let Ok(name) = device.name() {
                    name.contains(device_name)
                } else {
                    false
                }
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

    pub fn open_input_stream<F>(&self, callback: F) -> Result<Box<dyn Stream>, Box<dyn std::error::Error>>
    where
        F: FnMut(&[AudioSample]) + Send + 'static,
    {
        let device = self.input_device.as_ref().ok_or("No input device selected")?;
        
        // 获取设备支持的配置
        let config = device.default_input_config()
            .map_err(|_| "Failed to get default input config")?;
        
        let err_fn = |err| eprintln!("An error occurred on the input audio stream: {}", err);
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => {
                self.build_input_stream::<i16>(device, &config.into(), callback, err_fn)?
            }
            cpal::SampleFormat::U16 => {
                self.build_input_stream::<u16>(device, &config.into(), callback, err_fn)?
            }
            cpal::SampleFormat::F32 => {
                self.build_input_stream::<f32>(device, &config.into(), callback, err_fn)?
            }
        };
        
        Ok(Box::new(stream))
    }

    fn build_input_stream<T>(
        &self,
        device: &Device,
        config: &cpal::StreamConfig,
        mut callback: impl FnMut(&[AudioSample]) + Send + 'static,
        err_fn: impl Fn(cpal::StreamError) + Send + 'static,
    ) -> Result<cpal::Stream, Box<dyn std::error::Error>>
    where
        T: cpal::Sample,
        AudioSample: From<T>,
    {
        let channels = config.channels as usize;
        
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let samples: Vec<AudioSample> = data.iter()
                    .map(|&sample| sample.into())
                    .collect();
                
                callback(&samples);
            },
            err_fn,
            None, // None means default stream flags
        )?;
        
        Ok(stream)
    }

    pub fn open_output_stream(&self) -> Result<OutputStreamHandle, Box<dyn std::error::Error>> {
        let device = self.output_device.as_ref().ok_or("No output device selected")?;
        
        // 获取设备支持的配置
        let config = device.default_output_config()
            .map_err(|_| "Failed to get default output config")?;
        
        let err_fn = |err| eprintln!("An error occurred on the output audio stream: {}", err);
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => {
                self.build_output_stream::<i16>(device, &config.into(), err_fn)?
            }
            cpal::SampleFormat::U16 => {
                self.build_output_stream::<u16>(device, &config.into(), err_fn)?
            }
            cpal::SampleFormat::F32 => {
                self.build_output_stream::<f32>(device, &config.into(), err_fn)?
            }
        };
        
        Ok(OutputStreamHandle::new(stream))
    }

    fn build_output_stream<T>(
        &self,
        device: &Device,
        config: &cpal::StreamConfig,
        err_fn: impl Fn(cpal::StreamError) + Send + 'static,
    ) -> Result<cpal::Stream, Box<dyn std::error::Error>>
    where
        T: cpal::Sample,
        AudioSample: Into<T>,
    {
        let sample_rate = config.sample_rate.0;
        let channels = config.channels as usize;
        
        // 创建一个共享缓冲区用于输出
        let output_buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = output_buffer.clone();
        
        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let buffer = buffer_clone.lock().unwrap();
                let mut sample_iter = buffer.iter();
                
                for frame in data.chunks_mut(channels) {
                    let sample = sample_iter.next()
                        .copied()
                        .unwrap_or(0i16); // 默认静音值
                        
                    for sample_out in frame.iter_mut() {
                        *sample_out = sample.into();
                    }
                }
            },
            err_fn,
            None, // None means default stream flags
        )?;
        
        Ok(stream)
    }
}

// 输出流句柄，用于播放音频
pub struct OutputStreamHandle {
    stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<AudioSample>>>,
}

impl OutputStreamHandle {
    fn new(stream: cpal::Stream) -> Self {
        OutputStreamHandle {
            stream,
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.play()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.pause()?;
        Ok(())
    }

    pub fn play_audio(&self, audio_data: &[AudioSample]) -> Result<usize, Box<dyn std::error::Error>> {
        {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.clear();
            buffer.extend_from_slice(audio_data);
        }
        
        Ok(audio_data.len())
    }
}

impl Drop for PhysicalAudioDevice {
    fn drop(&mut self) {
        println!("PhysicalAudioDevice dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices() -> Result<(), Box<dyn std::error::Error>> {
        println!("Listing input devices:");
        let input_devices = PhysicalAudioDevice::list_input_devices()?;
        for (name, _) in input_devices {
            println!("  - {}", name);
        }

        println!("\nListing output devices:");
        let output_devices = PhysicalAudioDevice::list_output_devices()?;
        for (name, _) in output_devices {
            println!("  - {}", name);
        }

        Ok(())
    }
}
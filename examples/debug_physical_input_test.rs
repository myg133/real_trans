//! 调试版物理音频输入测试
//! 直接使用cpal库测试物理音频设备

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 物理音频设备调试测试 ===");
    
    // 获取主机
    let host = cpal::default_host();
    
    // 获取默认输入设备
    let device = host.default_input_device()
        .ok_or("No input device available")?;
    
    println!("Using default input device: \"{}\"", device.name()?);
    
    // 获取支持的配置
    let config = device.default_input_config()
        .map_err(|_| "Failed to get default input config")?;
    
    println!("Default input config: {:?}", config);
    
    // 获取具体配置参数
    let sample_rate = config.sample_rate();
    let channels = config.channels();
    
    println!("Sample rate: {} Hz", sample_rate.0);
    println!("Channels: {}", channels);
    println!("Sample format: {:?}", config.sample_format());
    
    // 创建一个共享的音频数据缓冲区
    let audio_samples = Arc::new(Mutex::new(Vec::new()));
    let audio_samples_clone = Arc::clone(&audio_samples);
    
    // 计数器用于跟踪回调调用
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = Arc::clone(&callback_count);
    
    // 根据采样格式创建流
    let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);
    
    let stream_config: cpal::StreamConfig = config.clone().into(); // 克隆 config
    
    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => {
            build_input_stream_typed::<i16>(
                &device,
                &stream_config,
                audio_samples_clone,
                callback_count_clone,
                err_fn,
            )?
        }
        cpal::SampleFormat::U16 => {
            build_input_stream_typed::<u16>(
                &device,
                &stream_config,
                audio_samples_clone,
                callback_count_clone,
                err_fn,
            )?
        }
        cpal::SampleFormat::F32 => {
            build_input_stream_typed::<f32>(
                &device,
                &stream_config,
                audio_samples_clone,
                callback_count_clone,
                err_fn,
            )?
        }
        _ => return Err(format!("Unsupported sample format: {:?}", config.sample_format()).into()),
    };
    
    println!("\n🎯 准备开始录制...");
    println!("按 Enter 键开始录制测试...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    // 启动流
    stream.play()?;
    println!("\n🎬 开始录制... 请说话5秒钟");
    
    // 记录开始时间
    let start_time = std::time::Instant::now();
    
    // 持续5秒
    while start_time.elapsed().as_secs() < 5 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // 每秒打印一次状态
        if start_time.elapsed().as_secs() % 1 == 0 {
            let count = *callback_count.lock().unwrap();
            println!("   已运行 {} 秒, 回调次数: {}", start_time.elapsed().as_secs(), count);
        }
    }
    
    // 停止流
    stream.pause()?;
    println!("\n⏹️ 录制结束");
    
    // 检查收集到的数据
    let samples = audio_samples.lock().unwrap();
    let count = callback_count.lock().unwrap();
    
    println!("\n📊 测试结果:");
    println!("   回调总次数: {}", *count);
    println!("   收集到的样本数: {}", samples.len());
    
    if samples.len() > 0 {
        println!("   前10个样本: {:?}", &samples[..std::cmp::min(10, samples.len())]);
        
        // 计算音量统计
        let max_amplitude = samples.iter()
            .map(|&x: &f32| x.abs())
            .fold(0.0_f32, |max: f32, x: f32| max.max(x));
        
        let avg_amplitude = samples.iter()
            .map(|&x: &f32| x.abs())
            .sum::<f32>() / samples.len() as f32;
        
        println!("   最大振幅: {:.6}", max_amplitude);
        println!("   平均振幅: {:.6}", avg_amplitude);
        
        if max_amplitude < 0.001 {
            println!("   ⚠️  检测到音量非常低，可能需要提高麦克风增益或靠近麦克风");
        }
        
        // 保存到文件
        let file = std::fs::File::create("debug_physical_input.raw")?;
        let mut writer = std::io::BufWriter::new(file);
        for &sample in samples.iter() {
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            writer.write_all(&sample_i16.to_le_bytes())?;
        }
        writer.flush()?;
        println!("💾 音频数据已保存到 debug_physical_input.raw ({})", samples.len());
    } else {
        println!("   ❌ 没有收集到任何音频数据");
        println!("   可能的原因:");
        println!("     - 设备选择错误");
        println!("     - 权限不足");
        println!("     - 麦克风被其他程序占用");
        println!("     - 音频驱动问题");
    }
    
    Ok(())
}

// 为特定类型构建输入流的辅助函数
fn build_input_stream_typed<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    audio_samples: Arc<Mutex<Vec<f32>>>,
    callback_count: Arc<Mutex<usize>>,
    err_fn: impl Fn(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, Box<dyn std::error::Error>>
where
    T: cpal::SizedSample + cpal::FromSample<f32> + Into<f32> + Send + 'static,
{
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            // 更新回调计数
            let mut count_guard = callback_count.lock().unwrap();
            *count_guard += 1;
            let current_count = *count_guard;
            drop(count_guard); // 释放锁
            
            // 转换样本并添加到缓冲区
            let mut samples = audio_samples.lock().unwrap();
            
            // 限制缓冲区大小以避免内存耗尽
            if samples.len() < 100000 {  // 限制为最多100k个样本
                for &sample in data.iter() {
                    let float_sample: f32 = sample.into();
                    samples.push(float_sample);
                }
                
                if current_count % 10 == 0 {
                    println!("   回调 #{}, 新增 {} 个样本, 总计 {}", 
                        current_count, data.len(), samples.len());
                }
            }
        },
        err_fn,
        None,
    )?;
    
    Ok(stream)
}
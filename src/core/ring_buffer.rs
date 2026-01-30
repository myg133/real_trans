use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// 高性能环形缓冲区，用于处理实时音频流
/// 
/// 该结构实现了线程安全的环形缓冲区，支持生产者-消费者模式，
/// 专为实时音频数据处理设计。
pub struct RingBuffer<T> {
    buffer: Arc<Mutex<VecDeque<T>>>,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    /// 创建一个新的环形缓冲区
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// 写入数据到缓冲区
    pub fn write(&self, data: &[T]) -> usize {
        let mut buffer = self.buffer.lock().unwrap();
        let writable = self.capacity - buffer.len();
        let to_write = std::cmp::min(data.len(), writable);
        
        for item in data.iter().take(to_write) {
            buffer.push_back(item.clone());
        }
        
        // 如果缓冲区满了，移除旧数据
        while buffer.len() > self.capacity {
            buffer.pop_front();
        }
        
        to_write
    }

    /// 从缓冲区读取数据
    pub fn read(&self, size: usize) -> Vec<T> {
        let mut buffer = self.buffer.lock().unwrap();
        let readable = std::cmp::min(size, buffer.len());
        let mut result = Vec::with_capacity(readable);
        
        for _ in 0..readable {
            if let Some(item) = buffer.pop_front() {
                result.push(item);
            }
        }
        
        result
    }

    /// 获取缓冲区中可读数据的大小
    pub fn readable_size(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// 获取缓冲区中可写空间的大小
    pub fn writable_size(&self) -> usize {
        self.capacity - self.buffer.lock().unwrap().len()
    }

    /// 清空缓冲区
    pub fn clear(&self) {
        self.buffer.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let rb = RingBuffer::new(10);
        
        // 写入数据
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(rb.write(&data), 5);
        
        // 检查可读数据
        assert_eq!(rb.readable_size(), 5);
        assert_eq!(rb.writable_size(), 5);
        
        // 读取数据
        let read_data = rb.read(3);
        assert_eq!(read_data, vec![1, 2, 3]);
        assert_eq!(rb.readable_size(), 2);
        
        // 检查溢出处理 - 此时缓冲区中有2个元素，可写空间是8
        // 写入11个元素，但只能写入8个（因为容量限制）
        let large_data = vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let written = rb.write(&large_data);
        assert_eq!(written, 8); // 只能写入8个，因为可写空间只有8
        assert_eq!(rb.readable_size(), 10); // 应该是满的（2个旧的+8个新的）
    }
}
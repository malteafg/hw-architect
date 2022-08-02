use wgpu::util::DeviceExt;
use wgpu::{Buffer, BufferSlice, BufferUsages, Device, Queue};

/// sizes are in number of bytes
pub struct DBuffer {
    label: String,
    alloc_size: u64,
    use_size: u64,
    buffer: Buffer,
    usage: BufferUsages,
}

impl DBuffer {
    pub fn new(label: &str, usage: BufferUsages, device: &Device) -> Self {
        let alloc_size = 64;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: alloc_size,
            usage,
            mapped_at_creation: true,
        });

        DBuffer {
            label: label.to_string(),
            alloc_size,
            use_size: 0,
            buffer,
            usage: usage | BufferUsages::COPY_DST,
        }
    }

    pub fn write(&mut self, queue: &Queue, device: &Device, data: &[u8]) {
        if self.alloc_size < data.len() as u64 || self.alloc_size > 2 * data.len() as u64 {
            self.alloc_buffer(device, data);
        } else {
            queue.write_buffer(&self.buffer, 0, data);
            self.use_size = data.len() as u64;
        }
    }

    pub fn get_buffer_slice(&self) -> Option<BufferSlice> {
        match self.use_size {
            0 => None,
            _ => Some(self.buffer.slice(..self.use_size)),
        }
    }

    fn alloc_buffer(&mut self, device: &Device, data: &[u8]) {
        let data_size = data.len() as u64;
        let new_size = 1 << (data_size as f32).log2().ceil() as u64;

        let mut empty_data = (0..(new_size - data_size)).map(|_| 0u8).collect::<Vec<_>>();

        let mut data = data.to_vec();
        data.append(&mut empty_data);

        self.alloc_size = new_size;
        self.use_size = data_size;
        self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&self.label),
            contents: bytemuck::cast_slice(&data),
            usage: self.usage,
        });
    }
}

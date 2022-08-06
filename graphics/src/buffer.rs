use anyhow::anyhow;
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

pub struct VIBuffer {
    vertex_buffer: DBuffer,
    index_buffer: DBuffer,
    num_indices: u32,
}

impl VIBuffer {
    pub fn new(label: &str, device: &Device) -> Self {
        let vertex_buffer = DBuffer::new(
            &("vertex_".to_owned() + label),
            BufferUsages::VERTEX,
            &device,
        );
        let index_buffer =
            DBuffer::new(&("index_".to_owned() + label), BufferUsages::INDEX, &device);

        VIBuffer {
            vertex_buffer,
            index_buffer,
            num_indices: 0,
        }
    }

    pub fn write(
        &mut self,
        queue: &Queue,
        device: &Device,
        vertices: &[u8],
        indices: &[u8],
        num_indices: u32,
    ) {
        self.vertex_buffer.write(&queue, &device, &vertices);
        self.index_buffer.write(&queue, &device, &indices);
        self.num_indices = num_indices;
    }

    pub fn get_buffer_slice(&self) -> anyhow::Result<(BufferSlice, BufferSlice)> {
        let vertices = self
            .vertex_buffer
            .get_buffer_slice()
            .ok_or(anyhow!("no contents in vertex buffer"))?;
        let indices = self
            .index_buffer
            .get_buffer_slice()
            .ok_or(anyhow!("no contents in index buffer"))?;
        Ok((vertices, indices))
    }

    pub fn get_num_indices(&self) -> u32 {
        self.num_indices
    }

    // pub fn get_vertices(&self) -> Option<BufferSlice> {
    //     self.vertex_buffer.get_buffer_slice()
    // }

    // pub fn get_indices(&self) -> Option<BufferSlice> {
    //     self.index_buffer.get_buffer_slice()
    // }

    // pub fn bind(&mut self, render_pass: &wgpu::RenderPass) -> anyhow::Result<()> {
    //     let vertices = self
    //         .vertex_buffer
    //         .get_buffer_slice()
    //         .ok_or(anyhow!("no contents in vertex buffer"))?;
    //     let indices = self
    //         .index_buffer
    //         .get_buffer_slice()
    //         .ok_or(anyhow!("no contents in index buffer"))?;
    //     render_pass.set_vertex_buffer(0, vertices);
    //     render_pass.set_index_buffer(indices, wgpu::IndexFormat::Uint32);

    //     Ok(())
    // }
}

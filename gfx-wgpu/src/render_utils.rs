use crate::resources::shaders;

use std::rc::Rc;

use wgpu::util::DeviceExt;

pub struct GfxInit {
    pub device: Rc<wgpu::Device>,
    pub queue: Rc<wgpu::Queue>,
    color_format: wgpu::TextureFormat,

    texture_bgl: wgpu::BindGroupLayout,
    camera_bgl: wgpu::BindGroupLayout,
    light_bgl: wgpu::BindGroupLayout,
    color_bgl: wgpu::BindGroupLayout,

    /// Used only to create the render pipelines.
    shader_map: shaders::ShaderMap,
}

impl GfxInit {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,

        texture_bgl: wgpu::BindGroupLayout,
        camera_bgl: wgpu::BindGroupLayout,
        light_bgl: wgpu::BindGroupLayout,
        color_bgl: wgpu::BindGroupLayout,

        shader_map: shaders::ShaderMap,
    ) -> Self {
        Self {
            device,
            queue,
            color_format,
            texture_bgl,
            camera_bgl,
            light_bgl,
            color_bgl,
            shader_map,
        }
    }

    pub fn device(&self) -> Rc<wgpu::Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Rc<wgpu::Queue> {
        self.queue.clone()
    }

    pub fn color_format(&self) -> wgpu::TextureFormat {
        self.color_format
    }

    pub fn texture_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bgl
    }

    pub fn camera_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bgl
    }

    pub fn light_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.light_bgl
    }

    pub fn color_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.color_bgl
    }

    pub fn shader(&self, shader: &str) -> &wgpu::ShaderModule {
        self.shader_map.get(shader).unwrap()
    }

    pub fn create_color(
        &self,
        init_color: gfx_api::colors::RGBAColor,
        buffer_name: &str,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&(buffer_name.to_owned() + "_color_buffer")),
                contents: bytemuck::cast_slice(&[init_color]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: self.color_bgl(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&(buffer_name.to_owned() + "_color_bind_group")),
        });
        (buffer, bind_group)
    }

    pub fn create_render_pipeline(
        &self,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader: &wgpu::ShaderModule,
        name: &str,
    ) -> wgpu::RenderPipeline {
        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&(name.to_owned() + "_pipeline_layout")),
                bind_group_layouts,
                push_constant_ranges: &[],
            });
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&(name.to_owned() + "_render_pipeline")),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: vertex_layouts,

                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: color_format,
                        blend: Some(wgpu::BlendState {
                            // color: wgpu::BlendComponent::REPLACE,
                            // alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

/// Represents the data needed to render on the gpu.
pub struct GfxHandle {
    pub device: Rc<wgpu::Device>,
    pub queue: Rc<wgpu::Queue>,

    pub camera_bg: wgpu::BindGroup,
    pub light_bg: Rc<wgpu::BindGroup>,

    pub light_uniform: LightUniform,
    pub light_buffer: wgpu::Buffer,
}

impl GfxHandle {
    pub fn new(
        gfx: &GfxInit,
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        camera_bg: wgpu::BindGroup,
    ) -> Self {
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bg = Rc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gfx.light_bgl(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        }));
        Self {
            device,
            queue,
            camera_bg,
            light_bg,
            light_uniform,
            light_buffer,
        }
    }
}

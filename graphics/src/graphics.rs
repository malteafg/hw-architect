use glam::*;
use utils::Mat4Utils;
use wgpu::util::DeviceExt;

use crate::vertex::Vertex;
use winit::{dpi::PhysicalSize, window::Window};

use common::camera;
use common::road::generator as road;

use crate::{
    buffer::{self, VIBuffer},
    model, resources, terrain, texture,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

// We need this for Rust to store our data correlctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Mat4::IDENTITY.to_4x4(),
        }
    }

    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.calc_pos().extend(1.0).into();
        self.view_proj =
            (OPENGL_TO_WGPU_MATRIX * projection.calc_matrix() * camera.calc_matrix()).to_4x4();
    }
}

// struct Instance {
//     position: cgmath::Vector3<f32>,
//     rotation: cgmath::Quaternion<f32>,
// }
//
// impl Instance {
//     fn to_raw(&self) -> InstanceRaw {
//         let model =
//             cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation);
//         InstanceRaw {
//             model: model.into(),
//             normal: cgmath::Matrix3::from(self.rotation).into(),
//         }
//     }
// }
//
// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// #[allow(dead_code)]
// struct InstanceRaw {
//     model: [[f32; 4]; 4],
//     normal: [[f32; 3]; 3],
// }
//
// impl Vertex for InstanceRaw {
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
//             // We need to switch from using a step mode of Vertex to Instance
//             // This means that our shaders will only change to use the next
//             // instance when the shader starts processing a new instance
//             step_mode: wgpu::VertexStepMode::Instance,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
//                     // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
//                     shader_location: 5,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
//                 // for each vec4. We don't have to do this in code though.
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     shader_location: 6,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
//                     shader_location: 7,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
//                     shader_location: 8,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
//                     shader_location: 9,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
//                     shader_location: 10,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
//                     shader_location: 11,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//             ],
//         }
//     }
// }
//
// const NUM_INSTANCES_PER_ROW: u32 = 1;

// lib.rs
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

pub struct GfxState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    // render_pipeline: wgpu::RenderPipeline,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    projection: camera::Projection,
    // instances: Vec<Instance>,
    // instance_buffer: buffer::DBuffer,
    depth_texture: texture::Texture,
    obj_model: model::Model,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    terrain_mesh: terrain::TerrainMesh,
    terrain_render_pipeline: wgpu::RenderPipeline,
    road_buffer: buffer::VIBuffer,
    road_markings_buffer: buffer::VIBuffer,
    road_tool_buffer: buffer::VIBuffer,
    road_render_pipeline: wgpu::RenderPipeline,
    road_color_bind_group: wgpu::BindGroup,
    road_markings_color_bind_group: wgpu::BindGroup,
    road_tool_color_bind_group: wgpu::BindGroup,
    // road_material: model::Material,
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    name: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(name),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
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
    })
}

impl GfxState {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // instance is a handle to the GPU in use
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // surface is the part of the window that we draw to
        let surface = unsafe { instance.create_surface(window) };

        // adapter is direct handle to graphics card to retrieve information about it
        // is locked to specific backend; to graphics cards yield 4 adapters on window 2 for Vulkan and 2 for DirectX
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // disable wgpu features that does not work for WebGL when building for WebGL
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let projection =
            camera::Projection::new(size.width, size.height, 45.0f32.to_radians(), 5.0, 2000.0);
        let camera_uniform = CameraUniform::new();
        // camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // const SPACE_BETWEEN: f32 = 3.0;
        // let instances = (0..NUM_INSTANCES_PER_ROW)
        //     .flat_map(|z| {
        //         (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        //             let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        //             let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        //
        //             let position = cgmath::Vector3 { x, y: 0.0, z };
        //
        //             let rotation = if position.is_zero() {
        //                 cgmath::Quaternion::from_axis_angle(
        //                     cgmath::Vector3::unit_z(),
        //                     cgmath::Deg(0.0),
        //                 )
        //             } else {
        //                 cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        //             };
        //
        //             Instance { position, rotation }
        //         })
        //     })
        //     .collect::<Vec<_>>();
        //
        // let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        // let mut instance_buffer =
        //     buffer::DBuffer::new("Instance Buffer", wgpu::BufferUsages::VERTEX, &device);
        // instance_buffer.write(&queue, &device, &bytemuck::cast_slice(&instance_data));

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let obj_model =
            resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

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

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // let render_pipeline = {
        //     let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("Render Pipeline Layout"),
        //         bind_group_layouts: &[
        //             &texture_bind_group_layout,
        //             &camera_bind_group_layout,
        //             &light_bind_group_layout,
        //         ],
        //         push_constant_ranges: &[],
        //     });
        //
        //     let shader = wgpu::ShaderModuleDescriptor {
        //         label: Some("Normal Shader"),
        //         source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        //     };
        //     create_render_pipeline(
        //         &device,
        //         &layout,
        //         config.format,
        //         Some(texture::Texture::DEPTH_FORMAT),
        //         &[model::ModelVertex::desc(), InstanceRaw::desc()],
        //         shader,
        //         "Render Pipeline",
        //     )
        // };

        let road_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("road_color_buffer"),
            contents: bytemuck::cast_slice(&[Vec4::new(0.12, 0.12, 0.12, 1.0)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let road_markings_color_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("road_markings_color_buffer"),
                contents: bytemuck::cast_slice(&[Vec4::new(0.95, 0.95, 0.95, 1.0)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let road_tool_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("road_tool_color_buffer"),
            contents: bytemuck::cast_slice(&[Vec4::new(0.1, 0.1, 0.6, 0.5)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let road_color_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("road_color_bind_group_layout"),
            });

        let road_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &road_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: road_color_buffer.as_entire_binding(),
            }],
            label: Some("road_color_bind_group"),
        });

        let road_markings_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &road_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: road_markings_color_buffer.as_entire_binding(),
            }],
            label: Some("road_markings_color_bind_group"),
        });

        let road_tool_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &road_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: road_tool_color_buffer.as_entire_binding(),
            }],
            label: Some("road_color_bind_group"),
        });

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
                "Light Pipeline",
            )
        };

        let terrain_mesh = terrain::TerrainMesh::new(&device);
        let terrain_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("terrain_pipeline_layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("terrain_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("terrain.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[terrain::TerrainVertex::desc()],
                shader,
                "terrain_pipeline",
            )
        };

        let road_buffer = VIBuffer::new("road_buffer", &device);
        let road_markings_buffer = VIBuffer::new("road_markings_buffer", &device);
        let road_tool_buffer = VIBuffer::new("road_tool_buffer", &device);
        let road_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("road_pipeline_layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &road_color_bind_group_layout,
                    //&texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("road_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("road.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[road::RoadVertex::desc()],
                shader,
                "road_render_pipeline",
            )
        };

        // let diffuse_bytes = loader::load_binary("road-diffuse.png").await.unwrap();
        // let diffuse_texture =
        //     texture::Texture::from_bytes(&device, &queue, &diffuse_bytes, "road_diffuse", false)
        //         .unwrap();
        // let normal_bytes = loader::load_binary("road-normal.png").await.unwrap();
        // let normal_texture =
        //     texture::Texture::from_bytes(&device, &queue, &normal_bytes, "road_normal", true)
        //         .unwrap();

        // let road_material = model::Material::new(
        //     &device,
        //     "road_material",
        //     diffuse_texture,
        //     normal_texture,
        //     &texture_bind_group_layout,
        // );

        Self {
            surface,
            device,
            queue,
            config,
            size,
            // render_pipeline,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            projection,
            // instances,
            // instance_buffer,
            depth_texture,
            obj_model,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            terrain_mesh,
            terrain_render_pipeline,
            road_buffer,
            road_markings_buffer,
            road_tool_buffer,
            road_render_pipeline,
            road_color_bind_group,
            road_markings_color_bind_group,
            road_tool_color_bind_group,
            // road_material,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 0.5,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            // render_pass.set_pipeline(&self.render_pipeline);

            // use model::DrawModel;
            // let mesh = &self.obj_model.meshes[0];
            // let material = &self.obj_model.materials[mesh.material];
            // render_pass.draw_mesh_instanced(
            //     mesh,
            //     material,
            //     0..self.instances.len() as u32,
            //     &self.camera_bind_group,
            // );
            use model::DrawLight;

            // render terrain
            render_pass.set_pipeline(&self.terrain_render_pipeline);
            render_pass.set_vertex_buffer(0, self.terrain_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.terrain_mesh.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw_indexed(0..self.terrain_mesh.size as u32, 0, 0..1);

            // render roads
            render_pass.set_pipeline(&self.road_render_pipeline);
            if let Ok((vertices, indices)) = self.road_buffer.get_buffer_slice() {
                render_pass.set_vertex_buffer(0, vertices);
                render_pass.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
                // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.road_color_bind_group, &[]);
                render_pass.draw_indexed(0..self.road_buffer.get_num_indices(), 0, 0..1);
            }
            if let Ok((vertices, indices)) = self.road_markings_buffer.get_buffer_slice() {
                render_pass.set_vertex_buffer(0, vertices);
                render_pass.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
                // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.road_markings_color_bind_group, &[]);
                render_pass.draw_indexed(0..self.road_markings_buffer.get_num_indices(), 0, 0..1);
            }
            if let Ok((vertices, indices)) = self.road_tool_buffer.get_buffer_slice() {
                render_pass.set_vertex_buffer(0, vertices);
                render_pass.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
                // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.road_tool_color_bind_group, &[]);
                render_pass.draw_indexed(0..self.road_tool_buffer.get_num_indices(), 0, 0..1);
            }

            // render light
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            // render instances
            // match self.instance_buffer.get_buffer_slice() {
            //     Some(buffer_slice) => {
            //         render_pass.set_vertex_buffer(1, buffer_slice);
            //         render_pass.set_pipeline(&self.render_pipeline);
            //         render_pass.draw_model_instanced(
            //             &self.obj_model,
            //             0..self.instances.len() as u32,
            //             &self.camera_bind_group,
            //             &self.light_bind_group,
            //         );
            //         //render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
            //     }
            //     None => {}
            // }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.projection.resize(new_size.width, new_size.height);
        }

        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    // pub fn add_instance(&mut self, position: cgmath::Vector3<f32>) {
    //     self.instances.push(Instance {
    //         position,
    //         rotation: math_utils::quart(
    //             Rad(std::f32::consts::PI / 4.0),
    //             Vector3::new(0.0, 1.0, 0.0),
    //         ),
    //     });
    //     let instance_data = self
    //         .instances
    //         .iter()
    //         .map(Instance::to_raw)
    //         .collect::<Vec<_>>();
    //     self.instance_buffer.write(
    //         &self.queue,
    //         &self.device,
    //         &bytemuck::cast_slice(&instance_data),
    //     );
    // }
    //
    // pub fn remove_instance(&mut self) {
    //     if self.instances.len() != 0 {
    //         self.instances.remove(0);
    //         let instance_data = self
    //             .instances
    //             .iter()
    //             .map(Instance::to_raw)
    //             .collect::<Vec<_>>();
    //         self.instance_buffer.write(
    //             &self.queue,
    //             &self.device,
    //             &bytemuck::cast_slice(&instance_data),
    //         );
    //     }
    // }

    pub fn update_road_buffer(&mut self, mesh: road::RoadMesh) {
        self.road_buffer.write(
            &self.queue,
            &self.device,
            bytemuck::cast_slice(&mesh.vertices),
            bytemuck::cast_slice(&mesh.indices),
            mesh.indices.len() as u32,
        );
    }

    pub fn update_road_tool_buffer(&mut self, mesh: road::RoadMesh) {
        self.road_tool_buffer.write(
            &self.queue,
            &self.device,
            bytemuck::cast_slice(&mesh.vertices),
            bytemuck::cast_slice(&mesh.indices),
            mesh.indices.len() as u32,
        );
    }

    pub fn update(&mut self, dt: instant::Duration, camera: &camera::Camera) {
        self.camera_uniform
            .update_view_proj(camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Update the light
        let old_position: Vec3 = self.light_uniform.position.into();
        self.light_uniform.position = (Quat::from_axis_angle(
            (0.0, 1.0, 0.0).into(),
            (60.0 * dt.as_secs_f32()).to_radians(),
        ) * old_position)
            .into();

        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }

    pub fn calc_ray(
        &self,
        camera: &camera::Camera,
        mouse_pos: common::input::MousePos,
    ) -> (Vec3, Vec3) {
        let screen_vec = Vec4::new(
            2.0 * mouse_pos.x as f32 / self.size.width as f32 - 1.0,
            1.0 - 2.0 * mouse_pos.y as f32 / self.size.height as f32,
            1.0,
            1.0,
        );
        let eye_vec = self
            .projection
            .calc_matrix()
            .inverse()
            // .expect("Unable to cast ray, projection")
            * screen_vec;
        let full_vec = camera
            .calc_matrix()
            .inverse()
            // .expect("Unable to cast ray, view")
            * Vec4::new(eye_vec.x, eye_vec.y, -1.0, 0.0);
        let processed_vec = Vec3::new(full_vec.x, full_vec.y, full_vec.z).normalize();

        (processed_vec, camera.calc_pos())
    }
}

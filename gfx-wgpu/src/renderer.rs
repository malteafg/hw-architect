mod road_renderer;
pub mod terrain_renderer;

use glam::*;
use utils::{Mat3Utils, Mat4Utils};
use wgpu::util::DeviceExt;

use crate::vertex::Vertex;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{buffer, model, resources, texture};
use std::collections::HashMap;
use std::rc::Rc;

use utils::id::SegmentId;

use gfx_api::InstanceRaw;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

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

struct Instance {
    position: Vec3,
    rotation: Quat,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let model = Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation);
        InstanceRaw {
            model: model.to_4x4(),
            normal: Mat3::from_quat(self.rotation).to_3x3(),
        }
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub struct GfxState {
    surface: wgpu::Surface,
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    window_width: u32,
    window_height: u32,
    projection: Projection,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    obj_model: model::Model,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    terrain_renderer: terrain_renderer::TerrainState,
    road_renderer: road_renderer::RoadState,
    sphere_render_pipeline: wgpu::RenderPipeline,
    instances: Vec<Instance>,
    instance_buffer: buffer::DBuffer,
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModule,
    name: &str,
) -> wgpu::RenderPipeline {
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

const NUM_INSTANCES_PER_ROW: u32 = 10;

impl GfxState {
    pub async fn new(window: &Window) -> Self {
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

        let device = Rc::new(device);
        let queue = Rc::new(queue);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);

        let window_width = size.width;
        let window_height = size.height;

        // load shaders
        let mut shaders = crate::shaders::load_shaders(&device);

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

        let projection = Projection::new(
            window_width,
            window_height,
            45.0f32.to_radians(),
            5.0,
            2000.0,
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[gfx_api::CameraView::default()]),
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

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let obj_model =
            resources::load_model("sphere", &device, &queue, &texture_bind_group_layout)
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

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shaders.remove(crate::shaders::LIGHT).unwrap(),
                "Light Pipeline",
            )
        };

        let terrain_renderer = terrain_renderer::TerrainState::new(
            Rc::clone(&device),
            // Rc::clone(&queue),
            config.format,
            shaders.remove(crate::shaders::TERRAIN).unwrap(),
            &camera_bind_group_layout,
        );
        let road_renderer = road_renderer::RoadState::new(
            Rc::clone(&device),
            Rc::clone(&queue),
            config.format,
            shaders.remove(crate::shaders::ROAD).unwrap(),
            &camera_bind_group_layout,
        );

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = Vec3 { x, y: 0.0, z };

                    let rotation = if position == Vec3::ZERO {
                        Quat::from_axis_angle(Vec3::Z, 0.0)
                    } else {
                        Quat::from_axis_angle(position.normalize(), std::f32::consts::PI / 4.)
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let mut instance_buffer =
            buffer::DBuffer::new("instance_buffer", wgpu::BufferUsages::VERTEX, &device);
        instance_buffer.write(&queue, &device, &bytemuck::cast_slice(&instance_data));

        let sphere_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sphere_pipeline_layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shaders.remove(crate::shaders::BASIC).unwrap(),
                "sphere_renderer",
            )
        };

        Self {
            surface,
            device,
            queue,
            config,
            window_width,
            window_height,
            projection,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            obj_model,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            terrain_renderer,
            road_renderer,
            sphere_render_pipeline,
            instances,
            instance_buffer,
        }
    }
}

impl gfx_api::Gfx for GfxState {
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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

            use terrain_renderer::RenderTerrain;
            render_pass.render_terrain(&self.terrain_renderer, &self.camera_bind_group);

            // render light
            use model::DrawLight;
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            use road_renderer::RenderRoad;
            render_pass.render_roads(&self.road_renderer, &self.camera_bind_group);

            use model::DrawModel;
            // let mesh = &self.obj_model.meshes[0];
            // let material = &self.obj_model.materials[mesh.material];
            // render_pass.draw_mesh_instanced(
            //     mesh,
            //     material,
            //     0..self.instances.len() as u32,
            //     &self.camera_bind_group,
            //     &self.light_bind_group,
            // );

            match self.instance_buffer.get_buffer_slice() {
                Some(buffer_slice) => {
                    render_pass.set_vertex_buffer(1, buffer_slice);
                    render_pass.set_pipeline(&self.sphere_render_pipeline);
                    render_pass.draw_model_instanced(
                        &self.obj_model,
                        0..self.instances.len() as u32,
                        &self.camera_bind_group,
                        &self.light_bind_group,
                    );
                    //render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
                }
                None => {}
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

        self.projection.resize(new_size.width, new_size.height);
    }

    fn update(&mut self, dt: instant::Duration) {
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

    fn add_instance(&mut self, position: Vec3) {
        let rotation = if position == Vec3::ZERO {
            Quat::from_axis_angle(Vec3::Z, 0.0)
        } else {
            Quat::from_axis_angle(position.normalize(), std::f32::consts::PI / 4.)
        };
        self.instances.push(Instance { position, rotation });
        let instance_data = self
            .instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        self.instance_buffer.write(
            &self.queue,
            &self.device,
            &bytemuck::cast_slice(&instance_data),
        );
    }

    fn remove_instance(&mut self) {
        if self.instances.len() != 0 {
            self.instances.remove(0);
            let instance_data = self
                .instances
                .iter()
                .map(Instance::to_raw)
                .collect::<Vec<_>>();
            self.instance_buffer.write(
                &self.queue,
                &self.device,
                &bytemuck::cast_slice(&instance_data),
            );
        }
    }
}

use gfx_api::Camera;
use gfx_api::RoadMesh;

/// This implementation simply passes the information along to the road_renderer
impl gfx_api::GfxRoadData for GfxState {
    fn add_road_meshes(&mut self, meshes: HashMap<SegmentId, RoadMesh>) {
        self.road_renderer.add_road_meshes(meshes);
    }

    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>) {
        self.road_renderer.remove_road_meshes(ids);
    }

    fn set_road_tool_mesh(&mut self, road_mesh: Option<RoadMesh>) {
        self.road_renderer.set_road_tool_mesh(road_mesh);
    }

    fn mark_road_segments(&mut self, segments: Vec<SegmentId>) {
        self.road_renderer.mark_road_segments(segments)
    }
}

impl gfx_api::GfxCameraData for GfxState {
    fn update_camera(&mut self, camera: &Camera) {
        let view_pos = camera.calc_pos().extend(1.0).into();
        let view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.projection.calc_matrix() * camera.compute_view_matrix())
                .to_4x4();
        let camera_view = gfx_api::CameraView::new(view_pos, view_proj);
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_view]));
    }

    fn compute_ray(&self, mouse_pos: Vec2, camera: &Camera) -> utils::Ray {
        let screen_vec = Vec4::new(
            2.0 * mouse_pos.x as f32 / self.window_width as f32 - 1.0,
            1.0 - 2.0 * mouse_pos.y as f32 / self.window_height as f32,
            1.0,
            1.0,
        );
        let eye_vec = self.projection.calc_matrix().inverse() * screen_vec;
        let full_vec =
            camera.compute_view_matrix().inverse() * Vec4::new(eye_vec.x, eye_vec.y, -1.0, 0.0);
        let processed_vec = Vec3::new(full_vec.x, full_vec.y, full_vec.z).normalize();

        utils::Ray::new(camera.calc_pos(), processed_vec)
    }
}

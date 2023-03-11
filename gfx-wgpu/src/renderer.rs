mod road_renderer;
pub mod terrain_renderer;
mod tree_renderer;

// use crate::vertex::Vertex;
use crate::primitives;
use crate::resources;

use utils::id::SegmentId;
use utils::Mat4Utils;

use gfx_api::{GfxFrameError, RawCameraData};

use glam::*;
use wgpu::util::DeviceExt;

use std::collections::HashMap;
use std::rc::Rc;

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
    depth_texture: primitives::Texture,
    obj_model: primitives::Model,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    terrain_renderer: terrain_renderer::TerrainState,
    road_renderer: road_renderer::RoadState,
    tree_renderer: tree_renderer::TreeState,
}

fn create_render_pipeline(
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

impl GfxState {
    /// The units of the window sizes are in pixels and should be without the window decorations.
    pub async fn new<W>(window: &W, window_width: u32, window_height: u32) -> Self
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let mut timer = utils::time::Timer::new();

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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window_width,
            height: window_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);

        timer.emit("init_gfx_time");

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
            contents: bytemuck::cast_slice(&[CameraView::default()]),
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

        let depth_texture =
            primitives::Texture::create_depth_texture(&device, &config, "depth_texture");

        timer.emit("setup_time");

        // load shaders
        let mut shaders = crate::shaders::load_shaders(&device);

        timer.emit("shader_time");

        let obj_model =
            resources::load_model("sphere", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();
        timer.emit("obj_model");

        let test_model = resources::load_simple_model("simple_test", &device)
            .await
            .unwrap();
        timer.emit("test_model");

        let tree_model =
            resources::load_model("tree_test", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        timer.emit("tree_model");

        use primitives::Vertex;
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
                Some(primitives::Texture::DEPTH_FORMAT),
                &[primitives::ModelVertex::desc()],
                shaders.remove(crate::shaders::LIGHT).unwrap(),
                "Light Pipeline",
            )
        };

        timer.emit("light_time");

        let terrain_renderer = terrain_renderer::TerrainState::new(
            Rc::clone(&device),
            // Rc::clone(&queue),
            config.format,
            shaders.remove(crate::shaders::TERRAIN).unwrap(),
            &camera_bind_group_layout,
        );

        timer.emit("terrain_time");

        let road_renderer = road_renderer::RoadState::new(
            Rc::clone(&device),
            Rc::clone(&queue),
            config.format,
            shaders.remove(crate::shaders::ROAD).unwrap(),
            &camera_bind_group_layout,
            shaders.remove(crate::shaders::SIMPLE).unwrap(),
            test_model,
        );

        timer.emit("road_time");

        let tree_renderer = tree_renderer::TreeState::new(
            Rc::clone(&device),
            Rc::clone(&queue),
            config.format,
            &camera_bind_group_layout,
            &texture_bind_group_layout,
            &light_bind_group_layout,
            shaders.remove(crate::shaders::BASIC).unwrap(),
            tree_model,
        );

        timer.emit("tree_time");

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
            tree_renderer,
        }
    }
}

fn map_error(err: wgpu::SurfaceError) -> GfxFrameError {
    match err {
        wgpu::SurfaceError::Timeout => GfxFrameError::Timeout,
        wgpu::SurfaceError::Outdated => GfxFrameError::Outdated,
        wgpu::SurfaceError::Lost => GfxFrameError::Lost,
        wgpu::SurfaceError::OutOfMemory => GfxFrameError::OutOfMemory,
    }
}

impl gfx_api::Gfx for GfxState {
    fn render(&mut self) -> Result<(), GfxFrameError> {
        let output = self.surface.get_current_texture().map_err(map_error)?;
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
            use primitives::DrawLight;
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            use road_renderer::RenderRoad;
            render_pass.render_roads(&self.road_renderer, &self.camera_bind_group);

            use tree_renderer::RenderTrees;
            render_pass.render_trees(
                &self.tree_renderer,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 || height > 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture =
            primitives::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

        self.projection.resize(width, height);
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
}

use gfx_api::RoadMesh;

/// This implementation simply passes the information along to the road_renderer
impl gfx_api::GfxRoadData for GfxState {
    fn add_road_meshes(&mut self, meshes: HashMap<SegmentId, RoadMesh>) {
        self.road_renderer.add_road_meshes(meshes);
    }

    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>) {
        self.road_renderer.remove_road_meshes(ids);
    }

    fn mark_road_segments(&mut self, segments: Vec<SegmentId>) {
        self.road_renderer.mark_road_segments(segments)
    }

    fn set_road_tool_mesh(&mut self, road_mesh: Option<RoadMesh>) {
        self.road_renderer.set_road_tool_mesh(road_mesh);
    }

    fn set_node_markers(&mut self, markers: Vec<[f32; 3]>) {
        self.road_renderer.set_node_markers(markers);
    }
}

// Represents a cameras position and projection view matrix in raw form. It cannot be computed
// without the projection from the gpu side
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraView {
    view_pos: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            view_pos: [0.0; 4],
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl CameraView {
    pub fn new(view_pos: [f32; 4], view_proj: [[f32; 4]; 4]) -> Self {
        Self {
            view_pos,
            view_proj,
        }
    }
}

/// Computes and returns the camera's current view matrix
fn compute_view_matrix(camera: RawCameraData) -> Mat4 {
    let (sin_pitch, cos_pitch) = camera.pitch.sin_cos();
    let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();

    Mat4::look_to_rh(
        Vec3::from_array(camera.pos),
        Vec3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw).normalize(),
        Vec3::Y,
    )
}

impl gfx_api::GfxCameraData for GfxState {
    fn update_camera(&mut self, camera: RawCameraData) {
        let view_pos = Vec3::from_array(camera.pos).extend(1.0).into();
        let view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.projection.calc_matrix() * compute_view_matrix(camera))
                .to_4x4();
        let camera_view = CameraView::new(view_pos, view_proj);
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_view]));
    }

    fn compute_ray(&self, mouse_pos: [f32; 2], camera: RawCameraData) -> [f32; 3] {
        let screen_vec = Vec4::new(
            2.0 * mouse_pos[0] as f32 / self.window_width as f32 - 1.0,
            1.0 - 2.0 * mouse_pos[1] as f32 / self.window_height as f32,
            1.0,
            1.0,
        );
        let eye_vec = self.projection.calc_matrix().inverse() * screen_vec;
        let full_vec =
            compute_view_matrix(camera).inverse() * Vec4::new(eye_vec.x, eye_vec.y, -1.0, 0.0);
        let processed_vec = Vec3::new(full_vec.x, full_vec.y, full_vec.z).normalize();

        processed_vec.into()
    }
}

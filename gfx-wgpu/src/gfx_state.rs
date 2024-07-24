use crate::primitives;
use crate::render_utils;
use crate::render_utils::GfxInit;
use crate::renderer;
use crate::resources;

use gfx_api::GSegment;
use utils::id::{IdMap, SegmentId, TreeId};

use gfx_api::{colors, GfxError, RawCameraData, RoadMesh};

use std::rc::Rc;
use std::time::Duration;

pub struct GfxState<'a> {
    surface: wgpu::Surface<'a>,
    depth_texture: primitives::Texture,
    surface_config: wgpu::SurfaceConfiguration,

    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,

    camera: primitives::Camera,
    renderer: renderer::Renderer,
}

impl<'a> GfxState<'a> {
    /// The units of the window sizes are in pixels and should be without the window decorations.
    pub async fn new<W>(window: &'a W, window_width: u32, window_height: u32) -> Self
    where
        W: raw_window_handle::HasWindowHandle
            + raw_window_handle::HasDisplayHandle
            + wgpu::WasmNotSendSync,
    {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // surface is the part of the window that we draw to
        let surface = instance.create_surface(window).unwrap();

        // Adapter is direct handle to graphics card to retrieve information about it.
        // Is locked to specific backend; to graphics cards yield 4 adapters on window 2 for Vulkan
        // and 2 for DirectX
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
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let device = Rc::new(device);
        let queue = Rc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_width,
            height: window_height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let depth_texture =
            primitives::Texture::create_depth_texture(&device, &config, "depth_texture");

        let (texture_bgl, camera_bgl, light_bgl, color_bgl) =
            resources::create_bind_group_layouts(&device);

        // load everything
        let (shaders, simple_models, models) = resources::load_all(&device, &queue, &texture_bgl);
        let obj_model = resources::load_model("sphere", &device, &queue, &texture_bgl).unwrap();

        let camera = primitives::Camera::new(&device, config.width, config.height, &camera_bgl);

        let gfx = GfxInit::new(
            device.clone(),
            queue.clone(),
            config.format,
            texture_bgl,
            camera_bgl,
            light_bgl,
            color_bgl,
            camera.get_bind_group().clone(),
        );

        let main_renderer = renderer::Renderer::new(gfx, shaders, simple_models, models, obj_model);

        Self {
            surface,
            device,
            queue,
            depth_texture,
            surface_config: config,
            renderer: main_renderer,
            camera,
        }
    }
}

fn map_error(err: wgpu::SurfaceError) -> GfxError {
    match err {
        wgpu::SurfaceError::Timeout => GfxError::SurfaceTimeout,
        wgpu::SurfaceError::Outdated => GfxError::SurfaceOutdated,
        wgpu::SurfaceError::Lost => GfxError::SurfaceLost,
        wgpu::SurfaceError::OutOfMemory => GfxError::SurfaceOutOfMemory,
    }
}

impl<'a> gfx_api::Gfx for GfxState<'a> {
    fn render(&mut self) -> Result<(), GfxError> {
        let output = self.surface.get_current_texture().map_err(map_error)?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            use renderer::RenderMain;
            render_pass.render(&self.renderer);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if !(width > 0 && height > 0) {
            return;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        self.depth_texture = primitives::Texture::create_depth_texture(
            &self.device,
            &self.surface_config,
            "depth_texture",
        );

        self.camera.resize(width, height);
    }

    fn update(&mut self, dt: Duration) {
        self.renderer.update(dt, &self.queue);
    }
}

/// This implementation simply passes the information along to the road_renderer
impl<'a> gfx_api::GfxRoadData for GfxState<'a> {
    fn add_road_meshes(&mut self, meshes: IdMap<SegmentId, GSegment>) {
        self.renderer.road_renderer.add_road_meshes(meshes);
    }

    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>) {
        self.renderer.road_renderer.remove_road_meshes(ids);
    }

    fn mark_road_segments(&mut self, segments: Vec<SegmentId>) {
        self.renderer.road_renderer.mark_road_segments(segments)
    }

    fn set_road_tool_mesh(&mut self, road_mesh: Option<RoadMesh>) {
        self.renderer.road_renderer.set_road_tool_mesh(road_mesh);
    }

    fn set_node_markers(&mut self, markers: Vec<([f32; 3], [f32; 3])>) {
        self.renderer.road_renderer.set_node_markers(markers);
    }
}

impl<'a> gfx_api::GfxTreeData for GfxState<'a> {
    fn add_trees(&mut self, model_id: u128, trees: Vec<(TreeId, [f32; 3], f32)>) {
        self.renderer.tree_renderer.add_trees(model_id, trees);
    }

    fn remove_tree(&mut self, tree_id: TreeId, model_id: u128) {
        self.renderer.tree_renderer.remove_tree(tree_id, model_id);
    }

    fn set_tree_markers(&mut self, positions: Vec<[f32; 3]>, color: Option<colors::RGBAColor>) {
        self.renderer
            .tree_renderer
            .set_tree_markers(positions, color);
    }

    fn set_tree_tool(&mut self, model_id: u128, tree: Vec<([f32; 3], f32)>) {
        self.renderer.tree_renderer.set_tree_tool(model_id, tree);
    }
}

impl<'a> gfx_api::GfxCameraData for GfxState<'a> {
    fn update_camera(&mut self, camera: RawCameraData) {
        self.camera.update_camera(camera, &self.queue);
    }

    fn compute_ray(&self, mouse_pos: [f32; 2], camera: RawCameraData) -> [f32; 3] {
        self.camera.compute_ray(mouse_pos, camera)
    }
}

impl<'a> gfx_api::GfxCarData for GfxState<'a> {
    fn set_cars(&mut self, _pos_yrots: Vec<([f32; 3], f32)>) {}
}

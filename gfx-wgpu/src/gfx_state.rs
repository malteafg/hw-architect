use crate::primitives;
use crate::render_utils;
use crate::renderer;
use crate::resources;

use utils::id::{IdMap, SegmentId, TreeId};

use gfx_api::{colors, GfxFrameError, RawCameraData, RoadMesh};

use std::rc::Rc;
use std::time::Duration;

pub struct GfxState<'a> {
    surface: wgpu::Surface<'a>,
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    depth_texture: primitives::Texture,

    config: wgpu::SurfaceConfiguration,

    camera: primitives::Camera,
    main_renderer: renderer::Renderer,
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

        let (
            texture_bind_group_layout,
            camera_bind_group_layout,
            light_bind_group_layout,
            color_bind_group_layout,
        ) = resources::create_bind_group_layouts(&device);

        // load everything
        let (shaders, simple_models, models) =
            resources::load_all(&device, &queue, &texture_bind_group_layout);

        let obj_model =
            resources::load_model("sphere", &device, &queue, &texture_bind_group_layout).unwrap();

        let camera = primitives::Camera::new(
            &device,
            config.width,
            config.height,
            &camera_bind_group_layout,
        );

        let main_renderer = renderer::Renderer::new(
            Rc::clone(&device),
            Rc::clone(&queue),
            config.format,
            &texture_bind_group_layout,
            &camera_bind_group_layout,
            &light_bind_group_layout,
            &color_bind_group_layout,
            Rc::clone(camera.get_bind_group()),
            shaders,
            simple_models,
            models,
            obj_model,
        );

        Self {
            surface,
            device,
            queue,
            depth_texture,
            config,
            main_renderer,
            camera,
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

impl<'a> gfx_api::Gfx for GfxState<'a> {
    fn render(&mut self) -> Result<(), GfxFrameError> {
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
            render_pass.render_main(&self.main_renderer);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if !(width > 0 && height > 0) {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture =
            primitives::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

        self.camera.resize(width, height);
    }

    fn update(&mut self, dt: Duration) {
        self.main_renderer.update(dt, &self.queue);
    }
}

/// This implementation simply passes the information along to the road_renderer
impl<'a> gfx_api::GfxRoadData for GfxState<'a> {
    fn add_road_meshes(&mut self, meshes: IdMap<SegmentId, RoadMesh>) {
        self.main_renderer.road_renderer.add_road_meshes(meshes);
    }

    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>) {
        self.main_renderer.road_renderer.remove_road_meshes(ids);
    }

    fn mark_road_segments(&mut self, segments: Vec<SegmentId>) {
        self.main_renderer
            .road_renderer
            .mark_road_segments(segments)
    }

    fn set_road_tool_mesh(&mut self, road_mesh: Option<RoadMesh>) {
        self.main_renderer
            .road_renderer
            .set_road_tool_mesh(road_mesh);
    }

    fn set_node_markers(&mut self, markers: Vec<([f32; 3], [f32; 3])>) {
        self.main_renderer.road_renderer.set_node_markers(markers);
    }
}

impl<'a> gfx_api::GfxTreeData for GfxState<'a> {
    fn add_trees(&mut self, model_id: u128, trees: Vec<(TreeId, [f32; 3], f32)>) {
        self.main_renderer.tree_renderer.add_trees(model_id, trees);
    }

    fn remove_tree(&mut self, tree_id: TreeId, model_id: u128) {
        self.main_renderer
            .tree_renderer
            .remove_tree(tree_id, model_id);
    }

    fn set_tree_markers(&mut self, positions: Vec<[f32; 3]>, color: Option<colors::RGBAColor>) {
        self.main_renderer
            .tree_renderer
            .set_tree_markers(positions, color);
    }

    fn set_tree_tool(&mut self, model_id: u128, tree: Vec<([f32; 3], f32)>) {
        self.main_renderer
            .tree_renderer
            .set_tree_tool(model_id, tree);
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

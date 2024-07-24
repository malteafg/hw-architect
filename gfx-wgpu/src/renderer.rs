mod model_renderer;
mod road_renderer;
mod static_world_renderer;
mod terrain_renderer;
mod tree_renderer;

use crate::render_utils::*;
use crate::{primitives, renderer, resources};

use wgpu::util::DeviceExt;

use std::rc::Rc;
use std::time::Duration;

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
pub struct Renderer {
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_render_pipeline: wgpu::RenderPipeline,

    camera_bind_group: Rc<wgpu::BindGroup>,
    light_bind_group: Rc<wgpu::BindGroup>,

    terrain_renderer: terrain_renderer::TerrainState,
    pub road_renderer: road_renderer::RoadState,
    pub tree_renderer: tree_renderer::TreeState,

    simple_renderer: model_renderer::SimpleModelRenderer,
    model_renderer: model_renderer::ModelRenderer,

    /// temporary
    obj_model: primitives::Model,
}

impl Renderer {
    pub fn new(
        gfx: GfxInit,

        simple_model_map: resources::simple_models::SimpleModelMap,
        model_map: resources::models::ModelMap,

        obj_model: primitives::Model,
    ) -> Self {
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let light_bind_group = Rc::new(gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gfx.light_bgl(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        }));

        use primitives::Vertex;
        let light_render_pipeline = gfx.create_render_pipeline(
            &[gfx.camera_bgl(), gfx.light_bgl()],
            gfx.color_format(),
            Some(primitives::Texture::DEPTH_FORMAT),
            &[primitives::ModelVertex::desc()],
            gfx.shader(resources::shaders::LIGHT),
            "light",
        );

        let terrain_renderer = terrain_renderer::TerrainState::new(&gfx);
        let road_renderer = road_renderer::RoadState::new(&gfx);
        let tree_renderer = tree_renderer::TreeState::new(gfx.device(), gfx.queue());
        let simple_renderer = model_renderer::SimpleModelRenderer::new(&gfx, simple_model_map);
        let model_renderer =
            model_renderer::ModelRenderer::new(&gfx, model_map, Rc::clone(&light_bind_group));

        Self {
            light_uniform,
            light_buffer,
            light_render_pipeline,

            light_bind_group,
            camera_bind_group: gfx.camera_bg(),

            terrain_renderer,
            road_renderer,
            tree_renderer,

            simple_renderer,
            model_renderer,

            obj_model,
        }
    }

    pub fn update(&mut self, dt: Duration, queue: &wgpu::Queue) {
        // Update the light
        let old_position: glam::Vec3 = self.light_uniform.position.into();
        self.light_uniform.position = (glam::Quat::from_axis_angle(
            (0.0, 1.0, 0.0).into(),
            (60.0 * dt.as_secs_f32()).to_radians(),
        ) * old_position)
            .into();

        queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }
}

pub trait RenderMain<'a> {
    fn render(&mut self, renderer: &'a Renderer);
}

impl<'a, 'b> RenderMain<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render(&mut self, renderer: &'a Renderer) {
        use terrain_renderer::RenderTerrain;
        self.render_terrain(&renderer.terrain_renderer, &renderer.camera_bind_group);

        use primitives::DrawLight;
        self.set_pipeline(&renderer.light_render_pipeline);
        self.draw_light_model(
            &renderer.obj_model,
            &renderer.camera_bind_group,
            &renderer.light_bind_group,
        );

        use road_renderer::RenderRoad;
        self.render_roads(&renderer.road_renderer, &renderer.simple_renderer);

        use tree_renderer::RenderTrees;
        self.render_trees(
            &renderer.tree_renderer,
            &renderer.simple_renderer,
            &renderer.model_renderer,
        );
    }
}

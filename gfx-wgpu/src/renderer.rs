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

pub struct Renderer {
    light_render_pipeline: wgpu::RenderPipeline,

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
        gfx: &GfxInit,
        simple_model_map: resources::simple_models::SimpleModelMap,
        model_map: resources::models::ModelMap,
        obj_model: primitives::Model,
    ) -> Self {
        use primitives::Vertex;
        let light_render_pipeline = gfx.create_render_pipeline(
            &[gfx.camera_bgl(), gfx.light_bgl()],
            gfx.color_format(),
            Some(primitives::Texture::DEPTH_FORMAT),
            &[primitives::ModelVertex::desc()],
            gfx.shader(resources::shaders::LIGHT),
            "light",
        );

        let terrain_renderer = terrain_renderer::TerrainState::new(gfx);
        let road_renderer = road_renderer::RoadState::new(gfx);
        let tree_renderer = tree_renderer::TreeState::new(gfx.device(), gfx.queue());
        let simple_renderer = model_renderer::SimpleModelRenderer::new(gfx, simple_model_map);
        let model_renderer = model_renderer::ModelRenderer::new(gfx, model_map);

        Self {
            light_render_pipeline,

            terrain_renderer,
            road_renderer,
            tree_renderer,

            simple_renderer,
            model_renderer,

            obj_model,
        }
    }

    pub fn update(&mut self, dt: Duration, gfx_handle: &mut GfxHandle) {
        // Update the light
        let old_position: glam::Vec3 = gfx_handle.light_uniform.position.into();
        gfx_handle.light_uniform.position = (glam::Quat::from_axis_angle(
            (0.0, 1.0, 0.0).into(),
            (60.0 * dt.as_secs_f32()).to_radians(),
        ) * old_position)
            .into();

        gfx_handle.queue.write_buffer(
            &gfx_handle.light_buffer,
            0,
            bytemuck::cast_slice(&[gfx_handle.light_uniform]),
        );
    }
}

pub trait RenderMain<'a> {
    fn render(&mut self, gfx_handle: &'a GfxHandle, renderer: &'a Renderer);
}

impl<'a, 'b> RenderMain<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render(&mut self, gfx_handle: &'a GfxHandle, renderer: &'a Renderer) {
        use terrain_renderer::RenderTerrain;
        self.render_terrain(gfx_handle, &renderer.terrain_renderer);

        use primitives::DrawLight;
        self.set_pipeline(&renderer.light_render_pipeline);
        self.draw_light_model(
            &renderer.obj_model,
            &gfx_handle.camera_bg,
            &gfx_handle.light_bg,
        );

        use road_renderer::RenderRoad;
        self.render_roads(
            gfx_handle,
            &renderer.road_renderer,
            &renderer.simple_renderer,
        );

        use tree_renderer::RenderTrees;
        self.render_trees(
            gfx_handle,
            &renderer.tree_renderer,
            &renderer.simple_renderer,
            &renderer.model_renderer,
        );
    }
}

pub mod model_renderer;
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

    /// temporary
    obj_model: primitives::Model,
}

impl Renderer {
    pub fn new(gfx: &GfxInit, obj_model: primitives::Model) -> Self {
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

        Self {
            light_render_pipeline,
            terrain_renderer,
            road_renderer,
            tree_renderer,
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

impl<'a> StateRender<'a> for Renderer {
    fn render(&'a self, gfx_handle: &'a GfxHandle, render_pass: &mut wgpu::RenderPass<'a>) {
        self.terrain_renderer.render(gfx_handle, render_pass);

        use primitives::DrawLight;
        render_pass.set_pipeline(&self.light_render_pipeline);
        render_pass.draw_light_model(&self.obj_model, &gfx_handle.camera_bg, &gfx_handle.light_bg);

        self.road_renderer.render(gfx_handle, render_pass);
        self.tree_renderer.render(gfx_handle, render_pass);
    }
}

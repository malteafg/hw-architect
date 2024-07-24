use crate::primitives::{self, DBuffer, Instance, RoadBuffer, VIBuffer};
use crate::render_utils;
use crate::resources;

use super::model_renderer::{RenderSimpleModel, SimpleModelRenderer};
use super::{GfxHandle, GfxInit};

use utils::id::{IdMap, SegmentId};

use gfx_api::colors;
use gfx_api::{GSegment, RoadMesh};
use glam::*;

use std::rc::Rc;

pub struct RoadState {
    road_buffer: RoadBuffer,
    tool_buffer: RoadBuffer,
    marked_buffer: RoadBuffer,

    road_render_pipeline: wgpu::RenderPipeline,

    road_segments: IdMap<SegmentId, GSegment>,
    marked_meshes: Vec<SegmentId>,

    markers_buffer: DBuffer,
    markers_color: colors::RGBAColor,
    num_markers: u32,
}

impl RoadState {
    pub fn new(gfx: &GfxInit) -> Self {
        let (_, asphalt_color) =
            gfx.create_color(colors::rgba(colors::ASPHALT_COLOR, 1.0), "asphalt");
        let asphalt_color = Rc::new(asphalt_color);

        let (_, markings_color) =
            gfx.create_color(colors::rgba(colors::LANE_MARKINGS_COLOR, 1.0), "markings");
        let markings_color = Rc::new(markings_color);

        let (_, tool_color) = gfx.create_color(colors::rgba(colors::LIGHT_BLUE, 0.5), "asphalt");
        let tool_color = Rc::new(tool_color);

        let (_, marked_color) = gfx.create_color(colors::rgba(colors::RED, 0.7), "marked");
        let marked_color = Rc::new(marked_color);

        let road_buffer = RoadBuffer::new(
            gfx.device(),
            gfx.queue(),
            "road",
            asphalt_color,
            Rc::clone(&markings_color),
        );
        let tool_buffer = RoadBuffer::new(
            gfx.device(),
            gfx.queue(),
            "tool",
            tool_color,
            Rc::clone(&markings_color),
        );
        let marked_buffer = RoadBuffer::new(
            gfx.device(),
            gfx.queue(),
            "marked",
            marked_color,
            markings_color,
        );

        use primitives::Vertex;
        let road_render_pipeline = gfx.create_render_pipeline(
            &[
                gfx.camera_bgl(),
                gfx.color_bgl(),
                //&texture_bind_group_layout,
            ],
            gfx.color_format(),
            Some(primitives::Texture::DEPTH_FORMAT),
            &[<[f32; 3]>::desc()],
            gfx.shader(resources::shaders::ROAD),
            "road",
        );

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
        // )

        let markers_buffer = DBuffer::new(
            gfx.device(),
            gfx.queue(),
            "markers_buffer",
            wgpu::BufferUsages::VERTEX,
        );
        let markers_color = colors::rgba(colors::RED, 0.8);

        Self {
            road_buffer,
            tool_buffer,
            marked_buffer,
            road_render_pipeline,
            road_segments: IdMap::new(),
            marked_meshes: Vec::new(),
            markers_buffer,
            markers_color,
            num_markers: 0,
        }
    }

    /// Combines the road meshes that road renderer stores in memory, and writes this to the gpu.
    fn write_road_mesh(&mut self) {
        let all = self.road_segments.keys().fold(vec![], |mut acc, x| {
            if !self.marked_meshes.contains(&x) {
                acc.push(x);
            }
            acc
        });
        let road_mesh = combine_road_meshes(&self.road_segments, &all);
        self.road_buffer.write(road_mesh);

        let marked_mesh = combine_road_meshes(&self.road_segments, &self.marked_meshes);
        self.marked_buffer.write(marked_mesh);
    }
}

impl gfx_api::GfxRoadData for RoadState {
    /// Adds a set of road meshes to what is stored in memory, and updates the gpu road meshes
    /// buffer.
    fn add_road_meshes(&mut self, meshes: IdMap<SegmentId, GSegment>) {
        self.road_segments.extend(meshes);
        self.write_road_mesh();
    }

    /// Removes a set of road meshes given by their ids from what is stored in memory, and updates
    /// the gpu road meshes buffer.
    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>) {
        ids.iter().for_each(|id| {
            self.road_segments.remove(*id);
        });
        self.write_road_mesh();
    }

    fn mark_road_segments(&mut self, segments: Vec<SegmentId>) {
        self.marked_meshes = segments;
        self.write_road_mesh();
    }

    /// Updates the road tool buffer with the given mesh.
    fn set_road_tool_mesh(&mut self, mesh: Option<RoadMesh>) {
        let Some(mesh) = mesh else {
            self.tool_buffer.write(RoadMesh::default());
            return;
        };
        self.tool_buffer.write(mesh);
    }

    fn set_node_markers(&mut self, markers: Vec<([f32; 3], [f32; 3])>) {
        self.num_markers = markers.len() as u32;
        let instance_data = markers
            .into_iter()
            .map(|(pos, dir)| {
                let dir = Vec3::from_array(dir);
                let mat = Mat3::from_cols_array(&[dir.x, 0., dir.z, 0., 1., 0., -dir.z, 0., dir.x]);
                Instance::new(Vec3::from_array(pos), glam::Quat::from_mat3(&mat))
            })
            .collect::<Vec<_>>()
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        self.markers_buffer
            .write(&bytemuck::cast_slice(&instance_data));
    }
}

/// Iterates over the given road_meshes and returns a vec of {`RoadVertex`} for writing to the gpu.
fn combine_road_meshes(
    road_meshes: &IdMap<SegmentId, GSegment>,
    selected_segments: &Vec<SegmentId>,
) -> RoadMesh {
    let mut indices_count = 0;
    let mut road_mesh: RoadMesh = RoadMesh::default();

    for (id, segment) in road_meshes.iter() {
        if !selected_segments.contains(&id) {
            continue;
        }
        road_mesh
            .vertices
            .append(&mut segment.road_mesh.vertices.clone());
        road_mesh.indices.append(
            &mut segment
                .road_mesh
                .indices
                .clone()
                .into_iter()
                .map(|i| i + indices_count)
                .collect(),
        );
        indices_count += segment.road_mesh.vertices.len() as u32;
    }

    indices_count = 0;
    for (id, segment) in road_meshes.iter() {
        if !selected_segments.contains(&id) {
            continue;
        }
        road_mesh
            .lane_vertices
            .append(&mut segment.road_mesh.lane_vertices.clone());
        road_mesh.lane_indices.append(
            &mut segment
                .road_mesh
                .lane_indices
                .clone()
                .into_iter()
                .map(|i| i + indices_count)
                .collect(),
        );
        indices_count += segment.road_mesh.lane_vertices.len() as u32;
    }

    road_mesh
}

/// A trait used by the main renderer to render the roads.
pub trait RenderRoad<'a> {
    /// The function that implements rendering for roads.
    fn render_roads(
        &mut self,
        gfx_handle: &'a GfxHandle,
        road_state: &'a RoadState,
        simple_renderer: &'a SimpleModelRenderer,
    );
}

impl<'a, 'b> RenderRoad<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_roads(
        &mut self,
        gfx_handle: &'a GfxHandle,
        road_state: &'b RoadState,
        simple_renderer: &'a SimpleModelRenderer,
    ) {
        self.set_pipeline(&road_state.road_render_pipeline);
        self.set_bind_group(0, &gfx_handle.camera_bg, &[]);
        self.render(&road_state.road_buffer);
        self.render(&road_state.tool_buffer);
        self.render(&road_state.marked_buffer);

        self.render_simple_model(
            gfx_handle,
            simple_renderer,
            resources::simple_models::ARROW_MODEL,
            road_state.markers_color,
            &road_state.markers_buffer,
            road_state.num_markers,
        );
    }
}

trait RenderRoadBuffer<'a> {
    /// The function that implements rendering for a road buffer.
    fn render(&mut self, road_buffer: &'a RoadBuffer);
}

impl<'a, 'b> RenderRoadBuffer<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render(&mut self, road_buffer: &'b RoadBuffer) {
        if let Ok((vertices, indices)) = road_buffer.mesh_buffer.get_buffer_slice() {
            self.set_bind_group(1, &road_buffer.asphalt_color, &[]);
            self.set_vertex_buffer(0, vertices);
            self.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
            // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
            self.draw_indexed(0..road_buffer.mesh_buffer.get_num_indices(), 0, 0..1);
        }
        if let Ok((vertices, indices)) = road_buffer.lane_buffer.get_buffer_slice() {
            self.set_bind_group(1, &road_buffer.lane_color, &[]);
            self.set_vertex_buffer(0, vertices);
            self.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
            // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
            self.draw_indexed(0..road_buffer.lane_buffer.get_num_indices(), 0, 0..1);
        }
    }
}

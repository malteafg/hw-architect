use crate::primitives::{self, DBuffer, Instance, RoadBuffer, VIBuffer};
use crate::render_utils;
use crate::resources;

use super::model_renderer::{RenderSimpleModel, SimpleModelRenderer};

use utils::id::{IdMap, SegmentId};

use gfx_api::colors;
use gfx_api::RoadMesh;
use glam::*;

use std::rc::Rc;

pub struct RoadState {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    camera_bind_group: Rc<wgpu::BindGroup>,

    road_buffer: RoadBuffer,
    tool_buffer: RoadBuffer,
    marked_buffer: RoadBuffer,

    road_render_pipeline: wgpu::RenderPipeline,

    road_meshes: IdMap<SegmentId, RoadMesh>,
    marked_meshes: Vec<SegmentId>,

    markers_buffer: DBuffer,
    markers_color: colors::RGBAColor,
    num_markers: u32,
}

impl RoadState {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,
        road_shader: wgpu::ShaderModule,

        camera_bind_group: Rc<wgpu::BindGroup>,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        color_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let (_, asphalt_color) = render_utils::create_color(
            &device,
            &color_bind_group_layout,
            colors::rgba(colors::ASPHALT_COLOR, 1.0),
            "asphalt",
        );
        let asphalt_color = Rc::new(asphalt_color);
        let (_, markings_color) = render_utils::create_color(
            &device,
            &color_bind_group_layout,
            colors::rgba(colors::LANE_MARKINGS_COLOR, 1.0),
            "markings",
        );
        let markings_color = Rc::new(markings_color);
        let (_, tool_color) = render_utils::create_color(
            &device,
            &color_bind_group_layout,
            colors::rgba(colors::LIGHT_BLUE, 0.5),
            "asphalt",
        );
        let tool_color = Rc::new(tool_color);
        let (_, marked_color) = render_utils::create_color(
            &device,
            &color_bind_group_layout,
            colors::rgba(colors::RED, 0.7),
            "marked",
        );
        let marked_color = Rc::new(marked_color);

        let road_buffer =
            RoadBuffer::new(&device, "road", asphalt_color, Rc::clone(&markings_color));
        let tool_buffer = RoadBuffer::new(&device, "tool", tool_color, Rc::clone(&markings_color));
        let marked_buffer = RoadBuffer::new(&device, "marked", marked_color, markings_color);

        use primitives::Vertex;
        let road_render_pipeline = render_utils::create_render_pipeline(
            &device,
            &[
                camera_bind_group_layout,
                &color_bind_group_layout,
                //&texture_bind_group_layout,
            ],
            color_format,
            Some(primitives::Texture::DEPTH_FORMAT),
            &[<[f32; 3]>::desc()],
            road_shader,
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

        let markers_buffer = DBuffer::new("markers_buffer", wgpu::BufferUsages::VERTEX, &device);
        let markers_color = colors::rgba(colors::RED, 0.8);

        Self {
            device,
            queue,
            camera_bind_group,
            road_buffer,
            tool_buffer,
            marked_buffer,
            road_render_pipeline,
            road_meshes: IdMap::new(),
            marked_meshes: Vec::new(),
            markers_buffer,
            markers_color,
            num_markers: 0,
        }
    }

    /// Combines the road meshes that road renderer stores in memory, and writes this to the gpu.
    fn write_road_mesh(&mut self) {
        let all = self.road_meshes.keys().fold(vec![], |mut acc, x| {
            if !self.marked_meshes.contains(&x) {
                acc.push(x);
            }
            acc
        });
        let road_mesh = combine_road_meshes(&self.road_meshes, &all);
        self.road_buffer.write(&self.queue, &self.device, road_mesh);

        let marked_mesh = combine_road_meshes(&self.road_meshes, &self.marked_meshes);
        self.marked_buffer
            .write(&self.queue, &self.device, marked_mesh);
    }
}

impl gfx_api::GfxRoadData for RoadState {
    /// Adds a set of road meshes to what is stored in memory, and updates the gpu road meshes
    /// buffer.
    fn add_road_meshes(&mut self, meshes: IdMap<SegmentId, RoadMesh>) {
        self.road_meshes.extend(meshes);
        self.write_road_mesh();
    }

    /// Removes a set of road meshes given by their ids from what is stored in memory, and updates
    /// the gpu road meshes buffer.
    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>) {
        ids.iter().for_each(|id| {
            self.road_meshes.remove(*id);
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
            self.tool_buffer
                .write(&self.queue, &self.device, RoadMesh::default());
            return;
        };
        self.tool_buffer.write(&self.queue, &self.device, mesh);
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
        self.markers_buffer.write(
            &self.queue,
            &self.device,
            &bytemuck::cast_slice(&instance_data),
        );
    }
}

/// Iterates over the given road_meshes and returns a vec of {`RoadVertex`} for writing to the gpu.
fn combine_road_meshes(
    road_meshes: &IdMap<SegmentId, RoadMesh>,
    selected_segments: &Vec<SegmentId>,
) -> RoadMesh {
    let mut indices_count = 0;
    let mut road_mesh: RoadMesh = RoadMesh::default();

    for (id, mesh) in road_meshes.iter() {
        if !selected_segments.contains(&id) {
            continue;
        }
        road_mesh.vertices.append(&mut mesh.vertices.clone());
        road_mesh.indices.append(
            &mut mesh
                .indices
                .clone()
                .into_iter()
                .map(|i| i + indices_count)
                .collect(),
        );
        indices_count += mesh.vertices.len() as u32;
    }

    indices_count = 0;
    for (id, mesh) in road_meshes.iter() {
        if !selected_segments.contains(&id) {
            continue;
        }
        road_mesh
            .lane_vertices
            .append(&mut mesh.lane_vertices.clone());
        road_mesh.lane_indices.append(
            &mut mesh
                .lane_indices
                .clone()
                .into_iter()
                .map(|i| i + indices_count)
                .collect(),
        );
        indices_count += mesh.lane_vertices.len() as u32;
    }

    road_mesh
}

/// A trait used by the main renderer to render the roads.
pub trait RenderRoad<'a> {
    /// The function that implements rendering for roads.
    fn render_roads(&mut self, road_state: &'a RoadState, simple_renderer: &'a SimpleModelRenderer);
}

impl<'a, 'b> RenderRoad<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_roads(
        &mut self,
        road_state: &'b RoadState,
        simple_renderer: &'a SimpleModelRenderer,
    ) {
        self.set_pipeline(&road_state.road_render_pipeline);
        self.set_bind_group(0, &road_state.camera_bind_group, &[]);
        self.render(&road_state.road_buffer);
        self.render(&road_state.tool_buffer);
        self.render(&road_state.marked_buffer);

        self.render_simple_model(
            simple_renderer,
            &road_state.queue,
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

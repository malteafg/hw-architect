use crate::primitives;
use crate::primitives::{DBuffer, VIBuffer};

use utils::id::SegmentId;
use utils::{Mat3Utils, Mat4Utils};

// temporary, remove once proper road markings
use gfx_api::InstanceRaw;
use gfx_api::RoadMesh;
use glam::*;
use wgpu::util::DeviceExt;

use std::collections::HashMap;
use std::rc::Rc;

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

pub(super) struct RoadState {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    road_buffer: RoadBuffer,
    tool_buffer: RoadBuffer,
    marked_buffer: RoadBuffer,
    road_render_pipeline: wgpu::RenderPipeline,
    road_meshes: HashMap<SegmentId, RoadMesh>,
    marked_meshes: Vec<SegmentId>,
    sphere_render_pipeline: wgpu::RenderPipeline,
    simple_render_pipeline: wgpu::RenderPipeline,
    instance_buffer: DBuffer,
    num_markers: u32,
    simple_model: primitives::SimpleModel,
    simple_color: wgpu::BindGroup,
}

/// The information needed on gpu to render a set of road meshes.
struct RoadBuffer {
    pub mesh_buffer: VIBuffer,
    pub lane_buffer: VIBuffer,
    asphalt_color: Rc<wgpu::BindGroup>,
    lane_color: Rc<wgpu::BindGroup>,
}

impl<'a, 'b> RoadBuffer {
    fn new(
        device: &wgpu::Device,
        label: &str,
        asphalt_color: Rc<wgpu::BindGroup>,
        lane_color: Rc<wgpu::BindGroup>,
    ) -> Self {
        let mesh_buffer = VIBuffer::new(&(label.to_owned() + "_buffer"), &device);
        let lane_buffer = VIBuffer::new(&(label.to_owned() + "_markings_buffer"), &device);

        Self {
            mesh_buffer,
            lane_buffer,
            asphalt_color,
            lane_color,
        }
    }

    fn write(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, mesh: RoadMesh) {
        self.mesh_buffer.write(
            queue,
            device,
            bytemuck::cast_slice(&mesh.vertices),
            bytemuck::cast_slice(&mesh.indices),
            mesh.indices.len() as u32,
        );
        self.lane_buffer.write(
            queue,
            device,
            bytemuck::cast_slice(&mesh.lane_vertices),
            bytemuck::cast_slice(&mesh.lane_indices),
            mesh.lane_indices.len() as u32,
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

fn create_color_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    color: Vec4,
    buffer_name: &str,
) -> Rc<wgpu::BindGroup> {
    let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&(buffer_name.to_owned() + "_color_buffer")),
        contents: bytemuck::cast_slice(&[color]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    Rc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: color_buffer.as_entire_binding(),
        }],
        label: Some(&(buffer_name.to_owned() + "_color_bind_group")),
    }))
}

impl RoadState {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,
        road_shader: wgpu::ShaderModule,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        // the following parameters should be removed after simpler rendering of road markers.
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        light_bind_group_layout: &wgpu::BindGroupLayout,
        basic_shader: wgpu::ShaderModule,
        simple_shader: wgpu::ShaderModule,
        test_model: primitives::SimpleModel,
    ) -> Self {
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

        let asphalt_color = create_color_group(
            &device,
            &road_color_bind_group_layout,
            Vec4::new(0.12, 0.12, 0.12, 1.0),
            "asphalt",
        );
        let markings_color = create_color_group(
            &device,
            &road_color_bind_group_layout,
            Vec4::new(0.95, 0.95, 0.95, 1.0),
            "markings",
        );
        let tool_color = create_color_group(
            &device,
            &road_color_bind_group_layout,
            Vec4::new(0.1, 0.1, 0.6, 0.5),
            "asphalt",
        );
        let marked_color = create_color_group(
            &device,
            &road_color_bind_group_layout,
            Vec4::new(1.0, 0.0, 0.1, 0.7),
            "marked",
        );

        let road_buffer =
            RoadBuffer::new(&device, "road", asphalt_color, Rc::clone(&markings_color));
        let tool_buffer = RoadBuffer::new(&device, "tool", tool_color, Rc::clone(&markings_color));
        let marked_buffer = RoadBuffer::new(&device, "marked", marked_color, markings_color);

        use primitives::Vertex;
        let road_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("road_pipeline_layout"),
                bind_group_layouts: &[
                    camera_bind_group_layout,
                    &road_color_bind_group_layout,
                    //&texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            super::create_render_pipeline(
                &device,
                &layout,
                color_format,
                Some(primitives::Texture::DEPTH_FORMAT),
                &[gfx_api::RoadVertex::desc()],
                road_shader,
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
        // )

        let instance_buffer = DBuffer::new("instance_buffer", wgpu::BufferUsages::VERTEX, &device);

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
            super::create_render_pipeline(
                &device,
                &layout,
                color_format,
                Some(primitives::Texture::DEPTH_FORMAT),
                &[primitives::ModelVertex::desc(), InstanceRaw::desc()],
                basic_shader,
                "sphere_renderer",
            )
        };

        let simple_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("simple_pipeline_layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &road_color_bind_group_layout],
                push_constant_ranges: &[],
            });
            super::create_render_pipeline(
                &device,
                &layout,
                color_format,
                Some(primitives::Texture::DEPTH_FORMAT),
                &[primitives::SimpleModelVertex::desc(), InstanceRaw::desc()],
                simple_shader,
                "simple_renderer",
            )
        };

        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("simple_color_buffer"),
            contents: bytemuck::cast_slice(&[Vec4::new(1.0, 0.2, 0.8, 0.7)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let simple_color = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &road_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some("simple_color_bind_group"),
        });

        Self {
            device,
            queue,
            road_buffer,
            tool_buffer,
            marked_buffer,
            road_render_pipeline,
            road_meshes: HashMap::new(),
            marked_meshes: Vec::new(),
            sphere_render_pipeline,
            simple_render_pipeline,
            instance_buffer,
            num_markers: 0,
            simple_model: test_model,
            simple_color,
        }
    }

    /// Combines the road meshes that road renderer stores in memory, and writes this to the gpu.
    fn write_road_mesh(&mut self) {
        let all = self.road_meshes.keys().fold(vec![], |mut acc, x| {
            if !self.marked_meshes.contains(x) {
                acc.push(*x);
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
    fn add_road_meshes(&mut self, meshes: HashMap<SegmentId, RoadMesh>) {
        self.road_meshes.extend(meshes);
        self.write_road_mesh();
    }

    /// Removes a set of road meshes given by their ids from what is stored in memory, and updates
    /// the gpu road meshes buffer.
    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>) {
        ids.iter().for_each(|id| {
            self.road_meshes.remove(id);
        });
        self.write_road_mesh();
    }

    fn mark_road_segments(&mut self, segments: Vec<SegmentId>) {
        self.marked_meshes = segments;
        self.write_road_mesh();
    }

    /// Updates the road tool buffer with the given mesh.
    fn set_road_tool_mesh(&mut self, mesh: Option<RoadMesh>) {
        if let Some(mesh) = mesh {
            self.tool_buffer.write(&self.queue, &self.device, mesh);
        }
    }

    fn set_node_markers(&mut self, markers: Vec<Vec3>) {
        self.num_markers = markers.len() as u32;
        let instance_data = markers
            .into_iter()
            .map(|pos| Instance {
                position: pos,
                rotation: glam::Quat::IDENTITY,
            })
            .collect::<Vec<_>>()
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

/// Iterates over the given road_meshes and returns a vec of {`RoadVertex`} for writing to the gpu.
fn combine_road_meshes(
    road_meshes: &HashMap<SegmentId, RoadMesh>,
    selected_segments: &Vec<SegmentId>,
) -> RoadMesh {
    let mut indices_count = 0;
    let mut road_mesh: RoadMesh = RoadMesh::default();

    for (id, mesh) in road_meshes.iter() {
        if !selected_segments.contains(id) {
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
        if !selected_segments.contains(id) {
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
pub(super) trait RenderRoad<'a> {
    /// The function that implements rendering for roads.
    fn render_roads(
        &mut self,
        road_state: &'a RoadState,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        obj_model: &'a primitives::Model,
    );
}

impl<'a, 'b> RenderRoad<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_roads(
        &mut self,
        road_state: &'b RoadState,
        camera_bind_group: &'b wgpu::BindGroup,
        // remove following parameters once road markers are properly implemented
        light_bind_group: &'b wgpu::BindGroup,
        obj_model: &'b primitives::Model,
    ) {
        self.set_pipeline(&road_state.road_render_pipeline);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.render(&road_state.road_buffer);
        self.render(&road_state.tool_buffer);
        self.render(&road_state.marked_buffer);

        // use super::model::DrawModel;
        // let mesh = &self.obj_model.meshes[0];
        // let material = &self.obj_model.materials[mesh.material];
        // render_pass.draw_mesh_instanced(
        //     mesh,
        //     material,
        //     0..self.instances.len() as u32,
        //     &self.camera_bind_group,
        //     &self.light_bind_group,
        // );

        // use super::model::DrawModel;
        // let Some(buffer_slice) = road_state.instance_buffer.get_buffer_slice() else {
        //     return;
        // };
        // self.set_vertex_buffer(1, buffer_slice);
        // self.set_pipeline(&road_state.sphere_render_pipeline);
        // self.draw_model_instanced(
        //     &obj_model,
        //     0..road_state.num_markers,
        //     &camera_bind_group,
        //     &light_bind_group,
        // );
        //render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

        use primitives::DrawSimpleModel;
        let Some(buffer_slice) = road_state.instance_buffer.get_buffer_slice() else {
            return;
        };
        self.set_vertex_buffer(1, buffer_slice);
        self.set_pipeline(&road_state.simple_render_pipeline);
        self.set_bind_group(1, &road_state.simple_color, &[]);
        self.draw_mesh_instanced(
            &road_state.simple_model,
            0..road_state.num_markers,
            &camera_bind_group,
        );
    }
}

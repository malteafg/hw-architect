use crate::primitives::{self, Model, SimpleModel};

use utils::loader;

use wgpu::util::DeviceExt;

use std::io::{BufReader, Cursor};

use std::collections::HashMap;

pub mod shaders {
    use super::*;
    pub type ShaderMap = HashMap<&'static str, wgpu::ShaderModule>;
    pub const BASIC: &str = "shader";
    pub const ROAD: &str = "road";
    pub const TERRAIN: &str = "terrain";
    pub const LIGHT: &str = "light";
    pub const SIMPLE: &str = "simple";
}
use shaders::ShaderMap;

pub mod simple_models {
    use super::*;
    pub type SimpleModelId = u128;
    pub type SimpleModelMap = HashMap<SimpleModelId, SimpleModel>;
    pub const TORUS_MODEL: SimpleModelId = 0;
    pub const ARROW_MODEL: SimpleModelId = 1;
    pub const SPHERE_MODEL: SimpleModelId = 2;
}
use simple_models::SimpleModelMap;

pub mod models {
    use super::*;
    pub type ModelId = u128;
    pub type ModelMap = HashMap<ModelId, Model>;
    pub const TREE_MODEL: ModelId = 0;
    // pub const SPEHRE_MODEL: ModelId = 1;
}
use models::ModelMap;

pub fn load_all(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> (ShaderMap, SimpleModelMap, ModelMap) {
    (
        load_shaders(device),
        load_simple_models(device),
        load_models(device, queue, texture_bind_group_layout),
    )
}

fn load_simple_models(device: &wgpu::Device) -> SimpleModelMap {
    let torus_model = load_simple_model("torus", &device).unwrap();
    let arrow_model = load_simple_model("arrow", &device).unwrap();
    let sphere_model = load_simple_model("sphere", &device).unwrap();

    let mut simple_models = HashMap::new();
    simple_models.insert(simple_models::TORUS_MODEL, torus_model);
    simple_models.insert(simple_models::ARROW_MODEL, arrow_model);
    simple_models.insert(simple_models::SPHERE_MODEL, sphere_model);
    simple_models
}

fn load_models(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
) -> ModelMap {
    let mut timer = utils::time::Timer::new();

    let tree_model = load_model("tree_test", &device, &queue, &texture_bind_group_layout).unwrap();
    timer.emit("tree_model");

    let mut models = HashMap::new();
    models.insert(models::TREE_MODEL, tree_model);

    models
}

fn load_shaders(device: &wgpu::Device) -> ShaderMap {
    let mut shaders = HashMap::new();

    load_shader(
        device,
        &mut shaders,
        shaders::BASIC,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
    );
    load_shader(
        device,
        &mut shaders,
        shaders::ROAD,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/road.wgsl").into()),
    );
    load_shader(
        device,
        &mut shaders,
        shaders::TERRAIN,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/terrain.wgsl").into()),
    );
    load_shader(
        device,
        &mut shaders,
        shaders::LIGHT,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
    );
    load_shader(
        device,
        &mut shaders,
        shaders::SIMPLE,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/simple.wgsl").into()),
    );

    shaders
}

pub fn load_texture(
    file_name: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<primitives::Texture> {
    let data = loader::load_binary(file_name)?;
    primitives::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
}

/// Loads 3D models from the res/models directory. To load the cube model for
/// instance simply pass "cube" as the file name
/// Note that this functions panics if a materials file cannot be found, which is not ideal.
pub fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<primitives::Model> {
    let mut timer = utils::time::Timer::new();

    let path = format!("models/{file_name}/");
    let obj_text = loader::load_string(&format!("{path}{file_name}.obj"))?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| {
            let p = p.to_str().unwrap();
            let file = p.replace(".mtl", "");
            let path = format!("models/{file}/{p}");
            let mat_text = loader::load_string(&path).unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )?;

    timer.emit("model_loaded");

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_path = format!("{}{}", path, m.diffuse_texture);
        let normal_path = format!("{}{}", path, m.normal_texture);
        let diffuse_texture = load_texture(&diffuse_path, false, device, queue)?;
        timer.emit("diffuse_loaded");
        let normal_texture = load_texture(&normal_path, true, device, queue)?;
        timer.emit("normal_loaded");

        materials.push(primitives::Material::new(
            device,
            &m.name,
            diffuse_texture,
            normal_texture,
            layout,
        ));
    }

    timer.emit("materials_loaded");

    use glam::*;

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| primitives::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    // We'll calculate these later
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: Vec3 = v0.position.into();
                let pos1: Vec3 = v1.position.into();
                let pos2: Vec3 = v2.position.into();

                let uv0: Vec2 = v0.tex_coords.into();
                let uv1: Vec2 = v1.tex_coords.into();
                let uv2: Vec2 = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent =
                    (tangent + Vec3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + Vec3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + Vec3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + Vec3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + Vec3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + Vec3::from(vertices[c[2] as usize].bitangent)).into();

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // Average the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (Vec3::from(v.tangent) * denom).into();
                v.bitangent = (Vec3::from(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            primitives::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    timer.emit("model_completed");
    Ok(primitives::Model { meshes, materials })
}

/// Will not load on web.
pub fn load_simple_model(
    file_name: &str,
    device: &wgpu::Device,
) -> anyhow::Result<primitives::SimpleModel> {
    // let path = format!("models/{file_name}/");
    // let obj_text = loader::load_string(&format!("{path}{file_name}.obj")).await?;
    // let obj_cursor = Cursor::new(obj_text);
    // let mut obj_reader = BufReader::new(obj_cursor);

    // let (models, obj_materials) =
    //     tobj::load_obj_buf_async(&mut obj_reader, &tobj::GPU_LOAD_OPTIONS, |p| async move {
    //         let file = p.replace(".mtl", "");
    //         let path = format!("models/{file}/{p}");
    //         let mat_text = loader::load_string(&path).await.unwrap();
    //         tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
    //     })
    //     .await?;

    let path = format!("res/models/{file_name}/");
    let test = tobj::load_obj(format!("{path}{file_name}.obj"), &tobj::GPU_LOAD_OPTIONS);

    let (models, _materials) = test.expect("Failed to load OBJ file");
    assert!(models.len() == 1);

    let obj_vertices = &models[0].mesh.positions;
    let obj_indices = &models[0].mesh.indices;
    let vertices: Vec<_> = (0..obj_vertices.len() / 3)
        .map(|i| primitives::SimpleModelVertex {
            position: [
                obj_vertices[i * 3],
                obj_vertices[i * 3 + 1],
                obj_vertices[i * 3 + 2],
            ],
        })
        .collect();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Vertex Buffer", file_name)),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Index Buffer", file_name)),
        contents: bytemuck::cast_slice(&obj_indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    Result::Ok(primitives::SimpleModel {
        name: file_name.to_string(),
        vertex_buffer,
        index_buffer,
        num_elements: obj_indices.len() as u32,
    })
}

fn load_shader(
    device: &wgpu::Device,
    shaders: &mut ShaderMap,
    name: &'static str,
    source: wgpu::ShaderSource,
) {
    let shader_name = format!("{}_shader", name);
    let shader_desc = wgpu::ShaderModuleDescriptor {
        label: Some(&shader_name),
        source,
    };
    shaders.insert(name, device.create_shader_module(shader_desc));
}

use wgpu::BindGroupLayout;

pub fn create_bind_group_layouts(
    device: &wgpu::Device,
) -> (
    BindGroupLayout,
    BindGroupLayout,
    BindGroupLayout,
    BindGroupLayout,
) {
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

    let color_bind_group_layout =
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
            label: Some("color_bind_group_layout"),
        });

    (
        texture_bind_group_layout,
        camera_bind_group_layout,
        light_bind_group_layout,
        color_bind_group_layout,
    )
}

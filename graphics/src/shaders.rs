use std::collections::HashMap;

pub type ShaderMap = HashMap<&'static str, wgpu::ShaderModule>;
pub const BASIC: &str = "shader";
pub const ROAD: &str = "road";
pub const TERRAIN: &str = "terrain";
pub const LIGHT: &str = "light";

pub fn load_shaders(device: &wgpu::Device) -> ShaderMap {
    let mut shaders = HashMap::new();

    load_shader(
        device,
        &mut shaders,
        BASIC,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
    );
    load_shader(
        device,
        &mut shaders,
        ROAD,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/road.wgsl").into()),
    );
    load_shader(
        device,
        &mut shaders,
        TERRAIN,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/terrain.wgsl").into()),
    );
    load_shader(
        device,
        &mut shaders,
        LIGHT,
        wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
    );

    shaders
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

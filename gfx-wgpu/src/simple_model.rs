use std::ops::Range;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleModelVertex {
    pub position: [f32; 3],
}

pub struct SimpleModel {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

pub trait DrawSimpleModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a SimpleModel, camera_bind_group: &'a wgpu::BindGroup);

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a SimpleModel,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawSimpleModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b SimpleModel, camera_bind_group: &'b wgpu::BindGroup) {
        self.draw_mesh_instanced(mesh, 0..1, camera_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b SimpleModel,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}

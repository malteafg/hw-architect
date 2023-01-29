//! This crate defines the api for the graphics engine that hw-architect uses. The only interaction
//! that other crates are allowed to have to a graphics engine must go through this api, to keep
//! things modular.

pub trait Gfx {
    // render should contain error handling as well
    // fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;

    // depends on winit
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);

    fn update(
        &mut self,
        gfx_data: &mut gfx_bridge::GfxData,
        dt: instant::Duration,
        camera_view: gfx_bridge::CameraView,
    );

    fn add_instance(&mut self, position: glam::Vec3);

    fn remove_instance(&mut self);
}

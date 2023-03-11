/// This enum represents the different errors that can occur when rendering the next frame. For now
/// this is a copy of wgpu::SurfaceError.
pub enum GfxFrameError {
    /// A timeout was encountered while trying to acquire the next frame.
    Timeout,
    /// The underlying surface has changed, and therefore the swap chain must be updated.
    Outdated,
    /// The swap chain has been lost and needs to be recreated.
    Lost,
    /// There is no more memory left to allocate a new frame.
    OutOfMemory,
}

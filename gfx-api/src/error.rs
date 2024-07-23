use thiserror::Error;

/// This enum represents the different errors that can occur when rendering graphics. Some errors
/// are mappings from wgpu::SurfaceError.
#[derive(Error, Debug)]
pub enum GfxError {
    #[error("A timeout was encountered while trying to acquire the next frame.")]
    SurfaceTimeout,

    #[error("The underlying surface has changed, and therefore the swap chain must be updated.")]
    SurfaceOutdated,

    #[error("The swap chain has been lost and needs to be recreated.")]
    SurfaceLost,

    #[error("There is no more memory left to allocate a new frame.")]
    SurfaceOutOfMemory,

    #[error("Attempting to get a slice of buffer `{0}` failed because the buffer is empty.")]
    BufferEmpty(String),

    #[error("Failed to load resource.")]
    LoadResourceFailed,
}

pub type GfxResult<T> = Result<T, GfxError>;

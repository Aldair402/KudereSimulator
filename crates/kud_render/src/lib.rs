//! Renderer abstraction and headless backend.
//!
//! The first backend is intentionally headless so simulation, assets, and tools
//! can be tested without a GPU. A Vulkan/wgpu backend can implement this trait.

use glam::{Mat4, Vec3};
use thiserror::Error;

/// A view/projection pair for one camera.
#[derive(Debug, Clone, Copy)]
pub struct CameraDesc {
    /// View matrix.
    pub view: Mat4,
    /// Projection matrix.
    pub projection: Mat4,
}

impl CameraDesc {
    /// Creates a simple perspective camera looking at the origin.
    pub fn looking_at_origin(aspect: f32) -> Self {
        Self {
            view: Mat4::look_at_rh(Vec3::new(0.0, 4.0, 10.0), Vec3::ZERO, Vec3::Y),
            projection: Mat4::perspective_rh_gl(60.0_f32.to_radians(), aspect, 0.1, 500.0),
        }
    }
}

/// One drawable submission.
#[derive(Debug, Clone, Copy)]
pub struct DrawItem {
    /// Stable mesh asset ID.
    pub mesh: u64,
    /// Stable material asset ID.
    pub material: u64,
    /// World transform.
    pub transform: Mat4,
}

/// Errors produced by render backends.
#[derive(Debug, Error)]
pub enum RenderError {
    /// The backend was not initialized.
    #[error("renderer is not initialized")]
    NotInitialized,

    /// A backend-specific failure occurred.
    #[error("render backend error: {0}")]
    Backend(String),
}

/// Minimal render backend contract.
pub trait RenderBackend {
    /// Prepares a frame.
    fn begin_frame(&mut self, camera: CameraDesc) -> Result<(), RenderError>;

    /// Submits one drawable.
    fn submit(&mut self, item: DrawItem) -> Result<(), RenderError>;

    /// Finishes a frame and returns the number of submitted draws.
    fn end_frame(&mut self) -> Result<u32, RenderError>;
}

/// A deterministic backend used by tests and headless simulation.
#[derive(Debug, Default)]
pub struct NullRenderer {
    draws: u32,
    in_frame: bool,
}

impl RenderBackend for NullRenderer {
    fn begin_frame(&mut self, _camera: CameraDesc) -> Result<(), RenderError> {
        self.draws = 0;
        self.in_frame = true;
        Ok(())
    }

    fn submit(&mut self, _item: DrawItem) -> Result<(), RenderError> {
        if !self.in_frame {
            return Err(RenderError::NotInitialized);
        }
        self.draws += 1;
        Ok(())
    }

    fn end_frame(&mut self) -> Result<u32, RenderError> {
        if !self.in_frame {
            return Err(RenderError::NotInitialized);
        }
        self.in_frame = false;
        Ok(self.draws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_renderer_counts_draws() {
        let mut renderer = NullRenderer::default();
        renderer
            .begin_frame(CameraDesc::looking_at_origin(1.0))
            .unwrap();
        renderer
            .submit(DrawItem {
                mesh: 1,
                material: 2,
                transform: Mat4::IDENTITY,
            })
            .unwrap();
        assert_eq!(renderer.end_frame().unwrap(), 1);
    }
}

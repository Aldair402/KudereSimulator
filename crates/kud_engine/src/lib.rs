//! Engine frame loop and subsystem orchestration.
//!
//! The engine owns fixed-step time, the simulation world, and render submission.
//! It does not depend on Unity or on a specific GPU API. Backends implement
//! `kud_render::RenderBackend`.

use kud_core::{CoreResult, FixedClock, FixedStep};
use kud_render::{CameraDesc, DrawItem, RenderBackend, RenderError};
use kud_sim::{GameEvent, Position, SimConfig, Simulation};

/// Engine configuration.
#[derive(Debug, Clone, Copy)]
pub struct EngineConfig {
    /// Fixed simulation frequency in hertz.
    pub fixed_hz: u32,
    /// Maximum fixed steps executed in one rendered frame.
    pub max_steps_per_frame: u32,
    /// Simulation configuration.
    pub sim: SimConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            fixed_hz: 60,
            max_steps_per_frame: 8,
            sim: SimConfig::default(),
        }
    }
}

/// Per-frame report returned by the engine.
#[derive(Debug, Clone)]
pub struct FrameReport {
    /// Number of fixed simulation steps executed.
    pub fixed_steps: u32,
    /// Interpolation alpha for rendering.
    pub alpha: f32,
    /// Events produced during this frame.
    pub events: Vec<GameEvent>,
}

/// Headless-capable engine instance.
pub struct Engine {
    config: EngineConfig,
    clock: FixedClock,
    /// Public simulation world for tools and tests.
    pub sim: Simulation,
    pending_events: Vec<GameEvent>,
    total_frames: u64,
}

impl Engine {
    /// Creates an engine from validated configuration.
    pub fn new(config: EngineConfig) -> CoreResult<Self> {
        let step = FixedStep::from_hz(config.fixed_hz)?;
        let clock = FixedClock::new(step, config.max_steps_per_frame)?;
        Ok(Self {
            config,
            clock,
            sim: Simulation::new(config.sim),
            pending_events: Vec::new(),
            total_frames: 0,
        })
    }

    /// Returns the engine configuration.
    pub fn config(&self) -> EngineConfig {
        self.config
    }

    /// Returns the total number of rendered frames.
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    /// Advances the engine by a variable frame delta.
    pub fn update(&mut self, frame_seconds: f32) -> CoreResult<FrameReport> {
        let steps = self.clock.advance(frame_seconds)?;
        let dt = self.clock.step().seconds();
        for _ in 0..steps {
            self.sim.tick(dt);
            self.sim.events.drain_previous(&mut self.pending_events);
        }
        self.total_frames += 1;
        Ok(FrameReport {
            fixed_steps: steps,
            alpha: self.clock.alpha(),
            events: std::mem::take(&mut self.pending_events),
        })
    }

    /// Submits the current simulation state to a render backend.
    pub fn render<R: RenderBackend>(
        &mut self,
        backend: &mut R,
        camera: CameraDesc,
    ) -> Result<u32, RenderError> {
        backend.begin_frame(camera)?;
        for (_, position) in self.sim.world.query::<&Position>().iter() {
            backend.submit(DrawItem {
                mesh: 1,
                material: 1,
                transform: glam::Mat4::from_translation(position.0),
            })?;
        }
        backend.end_frame()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kud_render::NullRenderer;

    #[test]
    fn update_emits_fixed_steps() {
        let mut engine = Engine::new(EngineConfig::default()).unwrap();
        let report = engine.update(1.0 / 60.0).unwrap();
        assert_eq!(report.fixed_steps, 1);
        assert_eq!(engine.sim.tick_count, 1);
    }

    #[test]
    fn headless_render_counts_positions() {
        let mut engine = Engine::new(EngineConfig::default()).unwrap();
        engine.sim.spawn_player(glam::Vec3::ZERO);
        engine
            .sim
            .spawn_student(1, glam::Vec3::X, kud_sim::Schedule::default())
            .unwrap();
        let mut renderer = NullRenderer::default();
        let draws = engine
            .render(&mut renderer, CameraDesc::looking_at_origin(1.0))
            .unwrap();
        assert_eq!(draws, 2);
    }
}

//! Deterministic fixed-step clock.
//!
//! The simulation advances at a constant tick rate. Rendering may run at a
//! variable rate and can use `alpha()` to interpolate between simulation states.

use crate::error::{CoreError, CoreResult};

/// A validated fixed simulation step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedStep {
    seconds: f32,
}

impl FixedStep {
    /// Creates a fixed step from a frequency in hertz.
    pub fn from_hz(hz: u32) -> CoreResult<Self> {
        if hz == 0 || hz > 240 {
            return Err(CoreError::InvalidConfig {
                field: "fixed_hz",
                reason: "must be between 1 and 240",
            });
        }
        Ok(Self {
            seconds: 1.0 / hz as f32,
        })
    }

    /// Returns the step duration in seconds.
    #[inline]
    pub fn seconds(self) -> f32 {
        self.seconds
    }
}

/// Accumulates variable frame time and emits fixed simulation steps.
#[derive(Debug, Clone)]
pub struct FixedClock {
    step: FixedStep,
    accumulator: f32,
    max_steps_per_frame: u32,
    total_steps: u64,
}

impl FixedClock {
    /// Creates a clock with a fixed step and a substep budget.
    pub fn new(step: FixedStep, max_steps_per_frame: u32) -> CoreResult<Self> {
        if max_steps_per_frame == 0 {
            return Err(CoreError::InvalidConfig {
                field: "max_steps_per_frame",
                reason: "must be greater than zero",
            });
        }
        Ok(Self {
            step,
            accumulator: 0.0,
            max_steps_per_frame,
            total_steps: 0,
        })
    }

    /// Returns the fixed step used by this clock.
    #[inline]
    pub fn step(&self) -> FixedStep {
        self.step
    }

    /// Returns the interpolation alpha in the range [0, 1).
    #[inline]
    pub fn alpha(&self) -> f32 {
        (self.accumulator / self.step.seconds()).clamp(0.0, 1.0)
    }

    /// Returns the total number of fixed steps emitted so far.
    #[inline]
    pub fn total_steps(&self) -> u64 {
        self.total_steps
    }

    /// Adds a variable frame delta and returns how many fixed steps to run.
    ///
    /// The accumulator is clamped to the configured substep budget to avoid a
    /// spiral of death after a breakpoint, load hitch, or window drag.
    pub fn advance(&mut self, frame_seconds: f32) -> CoreResult<u32> {
        if !frame_seconds.is_finite() || frame_seconds < 0.0 {
            return Err(CoreError::InvalidFrameDelta(frame_seconds));
        }

        self.accumulator += frame_seconds;
        let step_seconds = self.step.seconds();
        let mut steps = (self.accumulator / step_seconds).floor() as u32;

        if steps > self.max_steps_per_frame {
            steps = self.max_steps_per_frame;
            self.accumulator = step_seconds * self.max_steps_per_frame as f32;
        }

        self.accumulator -= steps as f32 * step_seconds;
        self.total_steps += u64::from(steps);
        Ok(steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_fixed_steps_at_60_hz() {
        let step = FixedStep::from_hz(60).unwrap();
        let mut clock = FixedClock::new(step, 8).unwrap();
        assert_eq!(clock.advance(1.0 / 60.0).unwrap(), 1);
        assert_eq!(clock.advance(1.0 / 60.0).unwrap(), 1);
        assert_eq!(clock.total_steps(), 2);
    }

    #[test]
    fn clamps_to_substep_budget() {
        let step = FixedStep::from_hz(60).unwrap();
        let mut clock = FixedClock::new(step, 4).unwrap();
        assert_eq!(clock.advance(1.0).unwrap(), 4);
        assert!(clock.alpha() < 1.0);
    }

    #[test]
    fn rejects_invalid_frame_delta() {
        let step = FixedStep::from_hz(60).unwrap();
        let mut clock = FixedClock::new(step, 4).unwrap();
        assert!(clock.advance(-0.1).is_err());
        assert!(clock.advance(f32::NAN).is_err());
    }
}

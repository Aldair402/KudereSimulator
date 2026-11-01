//! Deterministic fixed-step clock.
//!
//! The simulation advances at a constant tick rate. Rendering may run at a
//! variable rate and can use `alpha()` to interpolate between simulation states.
//!
//! Two independent limits protect the frame loop from a spiral of death:
//!
//! * `max_frame_delta` discards wall-clock time from a single long frame
//!   (breakpoint, window drag, asset load hitch) before it is accumulated.
//! * `max_steps_per_frame` caps how many simulation ticks one frame may run,
//!   so a slow machine degrades in simulation rate instead of freezing.
//!
//! Time discarded by either limit is tracked in `dropped_seconds` so tools can
//! report it instead of silently losing it.

use crate::error::{CoreError, CoreResult};

/// Upper bound accepted by [`FixedClock::set_time_scale`].
pub const MAX_TIME_SCALE: f32 = 8.0;

/// Default wall-clock delta accepted from a single frame, in seconds.
pub const DEFAULT_MAX_FRAME_DELTA: f32 = 0.25;

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
    max_frame_delta: f32,
    time_scale: f32,
    paused: bool,
    total_steps: u64,
    dropped_seconds: f32,
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
            max_frame_delta: DEFAULT_MAX_FRAME_DELTA,
            time_scale: 1.0,
            paused: false,
            total_steps: 0,
            dropped_seconds: 0.0,
        })
    }

    /// Returns the fixed step used by this clock.
    #[inline]
    pub fn step(&self) -> FixedStep {
        self.step
    }

    /// Returns the interpolation alpha in the range [0, 1].
    #[inline]
    pub fn alpha(&self) -> f32 {
        (self.accumulator / self.step.seconds()).clamp(0.0, 1.0)
    }

    /// Returns the total number of fixed steps emitted so far.
    #[inline]
    pub fn total_steps(&self) -> u64 {
        self.total_steps
    }

    /// Returns the total simulated time in seconds.
    ///
    /// Derived from the step count so it never drifts from the simulation.
    #[inline]
    pub fn simulated_seconds(&self) -> f32 {
        self.total_steps as f32 * self.step.seconds()
    }

    /// Returns wall-clock time discarded by the frame-delta or substep limits.
    #[inline]
    pub fn dropped_seconds(&self) -> f32 {
        self.dropped_seconds
    }

    /// Returns true when the clock emits no steps.
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Stops or resumes step emission without losing the accumulator.
    #[inline]
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Returns the current simulation time scale.
    #[inline]
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Sets the simulation time scale, in `0.0..=MAX_TIME_SCALE`.
    ///
    /// The scale changes how much simulated time a frame buys; it never changes
    /// the step duration, so simulation results stay reproducible.
    pub fn set_time_scale(&mut self, time_scale: f32) -> CoreResult<()> {
        if !time_scale.is_finite() || time_scale < 0.0 || time_scale > MAX_TIME_SCALE {
            return Err(CoreError::InvalidConfig {
                field: "time_scale",
                reason: "must be finite and between 0.0 and 8.0",
            });
        }
        self.time_scale = time_scale;
        Ok(())
    }

    /// Sets the largest wall-clock delta accepted from one frame.
    pub fn set_max_frame_delta(&mut self, max_frame_delta: f32) -> CoreResult<()> {
        if !max_frame_delta.is_finite() || max_frame_delta <= 0.0 {
            return Err(CoreError::InvalidConfig {
                field: "max_frame_delta",
                reason: "must be finite and greater than zero",
            });
        }
        self.max_frame_delta = max_frame_delta;
        Ok(())
    }

    /// Adds a variable frame delta and returns how many fixed steps to run.
    ///
    /// A paused clock validates its input but emits no steps and does not
    /// accumulate time, so resuming never produces a catch-up burst.
    pub fn advance(&mut self, frame_seconds: f32) -> CoreResult<u32> {
        if !frame_seconds.is_finite() || frame_seconds < 0.0 {
            return Err(CoreError::InvalidFrameDelta(frame_seconds));
        }
        if self.paused || self.time_scale == 0.0 {
            return Ok(0);
        }

        let accepted = frame_seconds.min(self.max_frame_delta);
        self.dropped_seconds += frame_seconds - accepted;
        self.accumulator += accepted * self.time_scale;

        let step_seconds = self.step.seconds();
        let mut steps = (self.accumulator / step_seconds).floor() as u32;

        if steps > self.max_steps_per_frame {
            let excess = self.accumulator - step_seconds * self.max_steps_per_frame as f32;
            self.dropped_seconds += excess.max(0.0) / self.time_scale;
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

    fn clock(hz: u32, budget: u32) -> FixedClock {
        FixedClock::new(FixedStep::from_hz(hz).unwrap(), budget).unwrap()
    }

    #[test]
    fn emits_fixed_steps_at_60_hz() {
        let mut clock = clock(60, 8);
        assert_eq!(clock.advance(1.0 / 60.0).unwrap(), 1);
        assert_eq!(clock.advance(1.0 / 60.0).unwrap(), 1);
        assert_eq!(clock.total_steps(), 2);
    }

    #[test]
    fn clamps_to_substep_budget() {
        let mut clock = clock(60, 4);
        assert_eq!(clock.advance(1.0).unwrap(), 4);
        assert!(clock.alpha() < 1.0);
        assert!(clock.dropped_seconds() > 0.0);
    }

    #[test]
    fn rejects_invalid_frame_delta() {
        let mut clock = clock(60, 4);
        assert!(clock.advance(-0.1).is_err());
        assert!(clock.advance(f32::NAN).is_err());
        assert!(clock.advance(f32::INFINITY).is_err());
    }

    #[test]
    fn pause_emits_no_steps_and_does_not_bank_time() {
        let mut clock = clock(60, 8);
        clock.set_paused(true);
        for _ in 0..120 {
            assert_eq!(clock.advance(1.0 / 60.0).unwrap(), 0);
        }
        assert_eq!(clock.total_steps(), 0);

        clock.set_paused(false);
        assert_eq!(clock.advance(1.0 / 60.0).unwrap(), 1);
        assert_eq!(clock.total_steps(), 1);
    }

    #[test]
    fn pause_still_validates_input() {
        let mut clock = clock(60, 8);
        clock.set_paused(true);
        assert!(clock.advance(-1.0).is_err());
    }

    #[test]
    fn time_scale_buys_more_steps_without_changing_step_size() {
        let mut clock = clock(60, 8);
        clock.set_time_scale(2.0).unwrap();
        let step_before = clock.step().seconds();
        assert_eq!(clock.advance(1.0 / 60.0).unwrap(), 2);
        assert_eq!(clock.step().seconds(), step_before);
    }

    #[test]
    fn zero_time_scale_freezes_the_simulation() {
        let mut clock = clock(60, 8);
        clock.set_time_scale(0.0).unwrap();
        assert_eq!(clock.advance(1.0).unwrap(), 0);
        assert_eq!(clock.total_steps(), 0);
    }

    #[test]
    fn rejects_out_of_range_time_scale() {
        let mut clock = clock(60, 8);
        assert!(clock.set_time_scale(-1.0).is_err());
        assert!(clock.set_time_scale(MAX_TIME_SCALE + 1.0).is_err());
        assert!(clock.set_time_scale(f32::NAN).is_err());
        assert_eq!(clock.time_scale(), 1.0);
    }

    #[test]
    fn frame_delta_limit_discards_hitches() {
        let mut clock = clock(60, 64);
        clock.set_max_frame_delta(0.1).unwrap();
        let steps = clock.advance(5.0).unwrap();
        assert_eq!(steps, 6);
        assert!((clock.dropped_seconds() - 4.9).abs() < 1e-4);
    }

    #[test]
    fn simulated_seconds_tracks_step_count() {
        let mut clock = clock(60, 8);
        for _ in 0..60 {
            clock.advance(1.0 / 60.0).unwrap();
        }
        assert_eq!(clock.total_steps(), 60);
        assert!((clock.simulated_seconds() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn step_sequence_is_reproducible() {
        let deltas = [0.016_f32, 0.021, 0.009, 0.033, 0.017, 0.004];
        let mut a = clock(60, 8);
        let mut b = clock(60, 8);
        for _ in 0..50 {
            for delta in deltas {
                assert_eq!(a.advance(delta).unwrap(), b.advance(delta).unwrap());
            }
        }
        assert_eq!(a.total_steps(), b.total_steps());
        assert_eq!(a.alpha().to_bits(), b.alpha().to_bits());
    }
}

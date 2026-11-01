//! Core primitives shared by the engine and the game simulation.
//!
//! This crate intentionally contains no rendering, windowing, or gameplay code.
//! It provides deterministic time stepping, configuration, and common errors.

pub mod error;
pub mod hash;
pub mod time;

pub use error::{CoreError, CoreResult};
pub use hash::StateHasher;
pub use time::{FixedClock, FixedStep, DEFAULT_MAX_FRAME_DELTA, MAX_TIME_SCALE};

/// A frame delta expressed in seconds.
pub type Seconds = f32;

/// Returns the project name used by logs and diagnostics.
pub const PROJECT_NAME: &str = "KudereSimulator";

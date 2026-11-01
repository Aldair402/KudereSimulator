//! Common error types for engine primitives.

use thiserror::Error;

/// Result alias used by core engine crates.
pub type CoreResult<T> = Result<T, CoreError>;

/// Errors that can be produced by core engine primitives.
#[derive(Debug, Error)]
pub enum CoreError {
    /// A configuration value is outside the supported range.
    #[error("invalid configuration for {field}: {reason}")]
    InvalidConfig {
        /// Name of the invalid field.
        field: &'static str,
        /// Human-readable explanation.
        reason: &'static str,
    },

    /// A frame delta was negative, NaN, or infinite.
    #[error("invalid frame delta: {0}")]
    InvalidFrameDelta(f32),

    /// A fixed-step simulation exceeded its substep budget.
    #[error("fixed-step budget exceeded: requested {requested}, max {max}")]
    FixedStepBudgetExceeded {
        /// Number of steps requested by the accumulator.
        requested: u32,
        /// Maximum number of steps allowed in one frame.
        max: u32,
    },
}

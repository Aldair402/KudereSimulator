//! Deterministic replay recording and verification.
//!
//! A replay stores the scenario setup plus one command per tick. Playing it back
//! against the same build must reproduce the recorded state hash exactly; any
//! divergence is a simulation regression, not a tolerance question.
//!
//! Format is versioned so an old replay fails loudly instead of silently
//! desynchronizing against a changed simulation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::input::InputCommand;
use crate::scenario::Scenario;
use crate::SimConfig;

/// Current replay format version.
pub const REPLAY_VERSION: u32 = 1;

/// Errors produced while reading or verifying a replay.
#[derive(Debug, Error)]
pub enum ReplayError {
    /// The replay uses a format this build cannot read.
    #[error("unsupported replay version {found}, expected {expected}")]
    UnsupportedVersion {
        /// Version stored in the file.
        found: u32,
        /// Version this build understands.
        expected: u32,
    },

    /// The replay declares a tick count that does not match its command list.
    #[error("replay declares {declared} ticks but contains {actual} commands")]
    TickCountMismatch {
        /// Tick count in the header.
        declared: u64,
        /// Number of recorded commands.
        actual: usize,
    },

    /// The simulation diverged from the recording.
    #[error(
        "replay desynchronized at tick {tick}: expected hash {expected:#018x}, got {actual:#018x}"
    )]
    Desynchronized {
        /// First tick whose hash differs.
        tick: u64,
        /// Hash stored in the replay.
        expected: u64,
        /// Hash produced by this build.
        actual: u64,
    },

    /// The replay file could not be parsed.
    #[error("replay JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// A recorded simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    /// Format version.
    pub version: u32,
    /// Fixed simulation frequency the recording was made at.
    pub fixed_hz: u32,
    /// RNG seed reserved for systems that need randomness.
    pub seed: u64,
    /// Scenario the recording was made against.
    pub scenario: Scenario,
    /// Simulation configuration the recording was made with.
    pub config: SimConfig,
    /// Number of ticks, equal to `commands.len()`.
    pub ticks: u64,
    /// One command per tick, in tick order.
    pub commands: Vec<InputCommand>,
    /// State hash after the final tick.
    pub final_hash: u64,
}

impl Replay {
    /// Creates a replay header for a recording in progress.
    pub fn new(fixed_hz: u32, seed: u64, scenario: Scenario, config: SimConfig) -> Self {
        Self {
            version: REPLAY_VERSION,
            fixed_hz,
            seed,
            scenario,
            config,
            ticks: 0,
            commands: Vec::new(),
            final_hash: 0,
        }
    }

    /// Appends one tick's command.
    pub fn record_tick(&mut self, command: InputCommand) {
        self.commands.push(command);
        self.ticks += 1;
    }

    /// Seals the recording with the final state hash.
    pub fn finish(&mut self, final_hash: u64) {
        self.final_hash = final_hash;
    }

    /// Validates header consistency before playback.
    pub fn validate(&self) -> Result<(), ReplayError> {
        if self.version != REPLAY_VERSION {
            return Err(ReplayError::UnsupportedVersion {
                found: self.version,
                expected: REPLAY_VERSION,
            });
        }
        if self.ticks != self.commands.len() as u64 {
            return Err(ReplayError::TickCountMismatch {
                declared: self.ticks,
                actual: self.commands.len(),
            });
        }
        Ok(())
    }

    /// Returns the command for a tick, defaulting to idle past the recording.
    pub fn command_at(&self, tick: u64) -> InputCommand {
        self.commands
            .get(tick as usize)
            .copied()
            .unwrap_or(InputCommand::Idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_scenario() -> Scenario {
        Scenario::SchoolDay { students: 4 }
    }

    #[test]
    fn replay_carries_its_scenario() {
        let mut replay = Replay::new(60, 0, test_scenario(), SimConfig::default());
        replay.record_tick(InputCommand::Idle);
        replay.finish(99);
        let json = serde_json::to_string(&replay).unwrap();
        let parsed: Replay = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.scenario, test_scenario());
        assert_eq!(parsed.config.minutes_per_tick, 1);
    }

    #[test]
    fn recording_counts_ticks() {
        let mut replay = Replay::new(60, 42, test_scenario(), SimConfig::default());
        replay.record_tick(InputCommand::Idle);
        replay.record_tick(InputCommand::Move { x: 127, z: 0 });
        replay.finish(0xdead_beef);
        assert_eq!(replay.ticks, 2);
        assert_eq!(replay.final_hash, 0xdead_beef);
        assert!(replay.validate().is_ok());
    }

    #[test]
    fn command_past_the_end_is_idle() {
        let replay = Replay::new(60, 0, test_scenario(), SimConfig::default());
        assert_eq!(replay.command_at(99), InputCommand::Idle);
    }

    #[test]
    fn version_mismatch_is_rejected() {
        let mut replay = Replay::new(60, 0, test_scenario(), SimConfig::default());
        replay.version = REPLAY_VERSION + 1;
        assert!(matches!(
            replay.validate(),
            Err(ReplayError::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn tick_count_mismatch_is_rejected() {
        let mut replay = Replay::new(60, 0, test_scenario(), SimConfig::default());
        replay.ticks = 5;
        assert!(matches!(
            replay.validate(),
            Err(ReplayError::TickCountMismatch { .. })
        ));
    }

    #[test]
    fn replay_round_trips_through_json() {
        let mut replay = Replay::new(60, 7, test_scenario(), SimConfig::default());
        replay.record_tick(InputCommand::Move { x: -50, z: 127 });
        replay.finish(123);
        let json = serde_json::to_string(&replay).unwrap();
        let parsed: Replay = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.commands, replay.commands);
        assert_eq!(parsed.final_hash, replay.final_hash);
        assert!(parsed.validate().is_ok());
    }
}

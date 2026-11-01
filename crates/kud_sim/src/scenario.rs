//! Reproducible simulation scenarios.
//!
//! A scenario is a deterministic setup function, not a saved world. Replays and
//! benchmarks reference it by name so both sides rebuild identical initial
//! state without shipping a scene file.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{Schedule, ScheduleBlock, SimConfig, Simulation, MINUTES_PER_DAY};

/// A named, versioned simulation setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scenario {
    /// Students on a grid walking to a shared destination area.
    Grid {
        /// Number of students to spawn.
        students: u32,
    },
    /// One school day with a small class group and phase transitions.
    SchoolDay {
        /// Number of students to spawn.
        students: u32,
    },
}

impl Scenario {
    /// Returns the canonical scenario name used in reports and replays.
    pub fn name(self) -> &'static str {
        match self {
            Self::Grid { .. } => "grid",
            Self::SchoolDay { .. } => "school_day",
        }
    }

    /// Builds the scenario into a fresh simulation.
    pub fn build(self, config: SimConfig) -> Result<Simulation, crate::SimError> {
        let mut sim = Simulation::new(config);
        match self {
            Self::Grid { students } => build_grid(&mut sim, students)?,
            Self::SchoolDay { students } => build_school_day(&mut sim, students)?,
        }
        Ok(sim)
    }
}

/// Spawns students on a square grid, each walking five meters forward.
fn build_grid(sim: &mut Simulation, students: u32) -> Result<(), crate::SimError> {
    sim.spawn_player(Vec3::ZERO);
    if students == 0 {
        return Ok(());
    }
    let side = (students as f32).sqrt().ceil() as u32;
    for id in 1..=students {
        let x = ((id - 1) % side) as f32 * 2.0;
        let z = ((id - 1) / side) as f32 * 2.0;
        let schedule = Schedule {
            blocks: vec![ScheduleBlock {
                start_minute: 0,
                end_minute: MINUTES_PER_DAY,
                destination: Vec3::new(x, 0.0, z + 5.0),
                action: "Walk".to_string(),
                phase: 1,
            }],
        };
        sim.spawn_student(id, Vec3::new(x, 0.0, z), schedule)?;
    }
    Ok(())
}

/// Spawns a class group that moves through locker, class, lunch, and club phases.
fn build_school_day(sim: &mut Simulation, students: u32) -> Result<(), crate::SimError> {
    sim.clock.minute = 7 * 60;
    sim.spawn_player(Vec3::new(0.0, 0.0, -20.0));
    if students == 0 {
        return Ok(());
    }
    let side = (students as f32).sqrt().ceil() as u32;
    for id in 1..=students {
        let column = ((id - 1) % side) as f32;
        let row = ((id - 1) / side) as f32;
        let schedule = Schedule {
            blocks: vec![
                ScheduleBlock {
                    start_minute: 420,
                    end_minute: 480,
                    destination: Vec3::new(column * 1.5, 0.0, 0.0),
                    action: "Locker".to_string(),
                    phase: 1,
                },
                ScheduleBlock {
                    start_minute: 480,
                    end_minute: 780,
                    destination: Vec3::new(column * 1.5, 0.0, 10.0 + row * 1.5),
                    action: "Class".to_string(),
                    phase: 2,
                },
                ScheduleBlock {
                    start_minute: 780,
                    end_minute: 840,
                    destination: Vec3::new(column * 2.0, 0.0, 25.0),
                    action: "Lunch".to_string(),
                    phase: 3,
                },
                ScheduleBlock {
                    start_minute: 840,
                    end_minute: 1020,
                    destination: Vec3::new(-10.0 + column * 2.0, 0.0, 40.0),
                    action: "Club".to_string(),
                    phase: 4,
                },
            ],
        };
        sim.spawn_student(id, Vec3::new(column * 1.5, 0.0, -5.0), schedule)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_spawns_requested_students_plus_player() {
        let sim = Scenario::Grid { students: 25 }
            .build(SimConfig::default())
            .unwrap();
        assert_eq!(sim.registry.len(), 25);
        assert_eq!(sim.world.len(), 26);
    }

    #[test]
    fn zero_students_is_valid() {
        let sim = Scenario::Grid { students: 0 }
            .build(SimConfig::default())
            .unwrap();
        assert!(sim.registry.is_empty());
    }

    #[test]
    fn scenario_build_is_reproducible() {
        let a = Scenario::SchoolDay { students: 10 }
            .build(SimConfig::default())
            .unwrap();
        let b = Scenario::SchoolDay { students: 10 }
            .build(SimConfig::default())
            .unwrap();
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn school_day_starts_before_first_period() {
        let sim = Scenario::SchoolDay { students: 4 }
            .build(SimConfig::default())
            .unwrap();
        assert_eq!(sim.clock.minute, 420);
    }

    #[test]
    fn scenario_names_are_stable() {
        assert_eq!(Scenario::Grid { students: 1 }.name(), "grid");
        assert_eq!(Scenario::SchoolDay { students: 1 }.name(), "school_day");
    }
}

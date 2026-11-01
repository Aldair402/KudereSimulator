//! Golden-hash regression tests for the simulation.
//!
//! These hashes are the contract for determinism. A change here means observable
//! simulation behavior changed. That is sometimes intended: when it is, update
//! the constant in the same commit and explain the behavior change in the
//! message. Never "fix" a failure by loosening the comparison.

use kud_sim::input::InputCommand;
use kud_sim::scenario::Scenario;
use kud_sim::{Replay, SimConfig, Simulation};

/// Fixed step used by every golden fixture.
const FIXTURE_HZ: u32 = 60;

/// Runs a scenario for a number of ticks with a scripted input pattern.
fn run(scenario: Scenario, ticks: u64, config: SimConfig) -> Simulation {
    let mut sim = scenario.build(config).expect("scenario builds");
    let dt = 1.0 / FIXTURE_HZ as f32;
    for tick in 0..ticks {
        sim.tick_with_input(dt, scripted_command(tick));
    }
    sim
}

/// Same input pattern the runner records, kept in sync deliberately.
fn scripted_command(tick: u64) -> InputCommand {
    match (tick / 60) % 5 {
        0 => InputCommand::Move { x: 127, z: 0 },
        1 => InputCommand::Move { x: 0, z: 127 },
        2 => InputCommand::Move { x: -127, z: 0 },
        3 => InputCommand::Move { x: 0, z: -127 },
        _ => InputCommand::Idle,
    }
}

#[test]
fn grid_100_students_600_ticks() {
    let sim = run(Scenario::Grid { students: 100 }, 600, SimConfig::default());
    assert_eq!(sim.state_hash(), 0x3edad586d9afee41);
}

#[test]
fn school_day_10_students_600_ticks() {
    let sim = run(
        Scenario::SchoolDay { students: 10 },
        600,
        SimConfig::default(),
    );
    assert_eq!(sim.state_hash(), 0x963cce80100e71e7);
}

#[test]
fn school_day_100_students_600_ticks() {
    let sim = run(
        Scenario::SchoolDay { students: 100 },
        600,
        SimConfig::default(),
    );
    assert_eq!(sim.state_hash(), 0xb8fe35e047415c5e);
}

#[test]
fn empty_scenario_is_stable() {
    let sim = run(Scenario::Grid { students: 0 }, 60, SimConfig::default());
    let again = run(Scenario::Grid { students: 0 }, 60, SimConfig::default());
    assert_eq!(sim.state_hash(), again.state_hash());
}

#[test]
fn replay_round_trip_preserves_the_hash() {
    let scenario = Scenario::SchoolDay { students: 10 };
    let config = SimConfig::default();
    let mut replay = Replay::new(FIXTURE_HZ, 0, scenario, config);
    let mut sim = scenario.build(config).expect("scenario builds");
    let dt = 1.0 / FIXTURE_HZ as f32;

    for tick in 0..600 {
        let command = scripted_command(tick);
        replay.record_tick(command);
        sim.tick_with_input(dt, command);
    }
    replay.finish(sim.state_hash());

    let json = serde_json::to_string(&replay).expect("serialize");
    let parsed: Replay = serde_json::from_str(&json).expect("deserialize");
    parsed.validate().expect("valid replay");

    let mut rebuilt = parsed.scenario.build(parsed.config).expect("rebuild");
    for tick in 0..parsed.ticks {
        rebuilt.tick_with_input(dt, parsed.command_at(tick));
    }
    assert_eq!(rebuilt.state_hash(), parsed.final_hash);
}

#[test]
fn different_tick_rates_produce_different_states() {
    let fast = run(Scenario::Grid { students: 10 }, 120, SimConfig::default());
    let slow = run(
        Scenario::Grid { students: 10 },
        120,
        SimConfig {
            minutes_per_tick: 2,
        },
    );
    assert_ne!(fast.state_hash(), slow.state_hash());
}

//! Tick-stamped input commands.
//!
//! Platform input is translated into commands before the simulation runs, so
//! the simulation never reads a keyboard, a clock, or a window. A replay is
//! just the recorded command stream, which is what makes it reproducible.
//!
//! Original reference: input reads scattered through `YandereScript.Update`.

use glam::Vec3;
use kud_ecs::World;
use serde::{Deserialize, Serialize};

use crate::{Player, Position};

/// Player movement speed in meters per second.
pub const PLAYER_SPEED: f32 = 3.5;

/// A simulation command applied at the start of a tick.
///
/// Directions are quantized to signed 8-bit axes so a recorded command is
/// byte-identical across platforms and cannot carry float noise from a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InputCommand {
    /// No input this tick.
    #[default]
    Idle,
    /// Move the player on the ground plane.
    Move {
        /// Strafe axis, -127..=127.
        x: i8,
        /// Forward axis, -127..=127.
        z: i8,
    },
}

impl InputCommand {
    /// Returns the normalized movement direction for this command.
    pub fn direction(self) -> Vec3 {
        match self {
            Self::Idle => Vec3::ZERO,
            Self::Move { x, z } => {
                let raw = Vec3::new(f32::from(x) / 127.0, 0.0, f32::from(z) / 127.0);
                if raw.length_squared() <= 1.0 {
                    raw
                } else {
                    raw.normalize()
                }
            }
        }
    }
}

/// Applies the tick's command to the player entity.
///
/// Original reference: `YandereScript` character movement.
pub fn input_system(world: &mut World, command: InputCommand, dt: f32) {
    let direction = command.direction();
    if direction == Vec3::ZERO {
        return;
    }
    for (_, (position, _)) in world.query_mut::<(&mut Position, &Player)>() {
        position.0 += direction * PLAYER_SPEED * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SimConfig, Simulation};

    #[test]
    fn idle_command_has_no_direction() {
        assert_eq!(InputCommand::Idle.direction(), Vec3::ZERO);
    }

    #[test]
    fn diagonal_input_is_normalized() {
        let direction = InputCommand::Move { x: 127, z: 127 }.direction();
        assert!((direction.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn partial_axis_input_is_not_amplified() {
        let direction = InputCommand::Move { x: 63, z: 0 }.direction();
        assert!(direction.length() < 0.5);
    }

    #[test]
    fn command_moves_the_player() {
        let mut sim = Simulation::new(SimConfig::default());
        let entity = sim.spawn_player(Vec3::ZERO);
        input_system(&mut sim.world, InputCommand::Move { x: 127, z: 0 }, 1.0);
        let position = sim.world.get::<&Position>(entity).unwrap().0;
        assert!((position.x - PLAYER_SPEED).abs() < 1e-4);
        assert_eq!(position.z, 0.0);
    }

    #[test]
    fn idle_command_does_not_move_the_player() {
        let mut sim = Simulation::new(SimConfig::default());
        let entity = sim.spawn_player(Vec3::X);
        input_system(&mut sim.world, InputCommand::Idle, 1.0);
        assert_eq!(sim.world.get::<&Position>(entity).unwrap().0, Vec3::X);
    }

    #[test]
    fn command_round_trips_through_json() {
        let command = InputCommand::Move { x: -127, z: 64 };
        let json = serde_json::to_string(&command).unwrap();
        assert_eq!(
            serde_json::from_str::<InputCommand>(&json).unwrap(),
            command
        );
    }
}

//! Simulation systems, run in a fixed order once per tick.
//!
//! Each system is a free function over the ECS world so it can be tested in
//! isolation and later scheduled in parallel where data access allows it.
//! Original reference: the `Update` bodies of `StudentScript` and
//! `StudentManagerScript`, split by responsibility instead of by MonoBehaviour.

use kud_ecs::World;

use crate::events::EventQueue;
use crate::{Facing, GameEvent, Perception, Player, Position, Schedule, Student};

/// Selects the active schedule block for each student.
///
/// Original reference: `StudentScript.Update` schedule/phase branch.
pub fn schedule_system(world: &mut World, minute: u16, events: &mut EventQueue) {
    for (_, (student, schedule)) in world.query_mut::<(&mut Student, &Schedule)>() {
        if !student.alive {
            continue;
        }
        let Some(block) = schedule.block_at(minute) else {
            continue;
        };
        student.destination = block.destination;
        if student.phase != block.phase {
            student.phase = block.phase;
            events.push(GameEvent::PhaseChanged {
                student_id: student.student_id,
                phase: block.phase,
            });
        }
    }
}

/// Moves students toward their current destination.
///
/// Original reference: `StudentScript` movement via `NavMeshAgent`. Arrival
/// snaps to the destination so repeated ticks cannot oscillate around it.
pub fn movement_system(world: &mut World, dt: f32) {
    for (_, (position, student)) in world.query_mut::<(&mut Position, &Student)>() {
        if !student.alive || student.speed <= 0.0 {
            continue;
        }
        let to_target = student.destination - position.0;
        let distance = to_target.length();
        if distance <= f32::EPSILON {
            continue;
        }
        let step = student.speed * dt;
        if step >= distance {
            position.0 = student.destination;
        } else {
            position.0 += to_target / distance * step;
        }
    }
}

/// Emits an event when a living student can see the player.
///
/// Original reference: `StudentScript` vision cone checks. Occlusion is not
/// modeled yet; the navigation raycast task adds it behind this same boundary.
pub fn perception_system(world: &mut World, events: &mut EventQueue) {
    let player_position = world
        .query::<(&Position, &Player)>()
        .iter()
        .next()
        .map(|(_, (position, _))| position.0);

    let Some(player_position) = player_position else {
        return;
    };

    for (_, (position, facing, perception, student)) in world
        .query::<(&Position, &Facing, &Perception, &Student)>()
        .iter()
    {
        if !student.alive {
            continue;
        }
        let offset = player_position - position.0;
        let distance = offset.length();
        if distance > perception.range || distance <= f32::EPSILON {
            continue;
        }
        let direction = offset / distance;
        if facing.0.normalize_or_zero().dot(direction) >= perception.fov_cos {
            events.push(GameEvent::StudentNoticedPlayer {
                student_id: student.student_id,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScheduleBlock, MINUTES_PER_DAY};
    use glam::Vec3;

    fn schedule_to(destination: Vec3, phase: u8) -> Schedule {
        Schedule {
            blocks: vec![ScheduleBlock {
                start_minute: 0,
                end_minute: MINUTES_PER_DAY,
                destination,
                action: "Walk".to_string(),
                phase,
            }],
        }
    }

    #[test]
    fn movement_snaps_on_arrival_and_stays() {
        let mut sim = crate::Simulation::new(crate::SimConfig::default());
        let entity = sim
            .spawn_student(1, Vec3::ZERO, schedule_to(Vec3::new(0.1, 0.0, 0.0), 1))
            .unwrap();
        for _ in 0..60 {
            sim.tick(1.0 / 60.0);
        }
        let position = sim.world.get::<&Position>(entity).unwrap().0;
        assert_eq!(position, Vec3::new(0.1, 0.0, 0.0));
    }

    #[test]
    fn dead_students_do_not_move_or_emit_phase_events() {
        let mut sim = crate::Simulation::new(crate::SimConfig::default());
        let entity = sim
            .spawn_student(1, Vec3::ZERO, schedule_to(Vec3::X * 50.0, 7))
            .unwrap();
        sim.world.get::<&mut Student>(entity).unwrap().alive = false;
        for _ in 0..30 {
            sim.tick(1.0 / 60.0);
        }
        assert_eq!(sim.world.get::<&Position>(entity).unwrap().0, Vec3::ZERO);
        assert!(sim.events.previous().is_empty());
    }

    #[test]
    fn perception_respects_range() {
        let mut sim = crate::Simulation::new(crate::SimConfig::default());
        sim.spawn_player(Vec3::new(0.0, 0.0, 500.0));
        sim.spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        sim.tick(1.0 / 60.0);
        assert!(sim.events.previous().is_empty());
    }

    #[test]
    fn perception_without_a_player_is_a_no_op() {
        let mut world = World::new();
        let mut events = EventQueue::new();
        perception_system(&mut world, &mut events);
        assert_eq!(events.current_len(), 0);
    }
}

//! Simulation systems, run in a fixed order once per tick.
//!
//! Each system is a free function over the ECS world so it can be tested in
//! isolation and later scheduled in parallel where data access allows it.
//! Original reference: the `Update` bodies of `StudentScript` and
//! `StudentManagerScript`, split by responsibility instead of by MonoBehaviour.
//!
//! Execution order per tick:
//!
//! 1. `input`      — apply the tick's player command
//! 2. `schedule`   — select the active timetable block
//! 3. `perception` — compute visibility and record sightings
//! 4. `report`     — act on suspicion reached during the previous tick
//! 5. `suspicion`  — integrate suspicion and decay memory
//! 6. `behavior`   — resolve the behavior state machine
//! 7. `movement`   — integrate position using the resolved state
//!
//! Reporting runs before decay so a student that reached the report threshold
//! cannot decay below it in the same tick and swallow the report.
//!
//! Perception runs before behavior so a state change reacts to this tick's
//! sighting, and movement runs last so it uses the resolved speed scale.

use glam::Vec3;
use kud_ecs::World;

use crate::behavior::{self, BehaviorState, Suspicion, TransitionInput};
use crate::events::EventQueue;
use crate::witness::{WitnessKind, WitnessMemory};
use crate::{Facing, GameEvent, Perception, Player, Position, Schedule, SchoolClock, Student};

/// Distance below which a student is considered to have arrived.
pub const ARRIVAL_EPSILON: f32 = 0.05;

/// Selects the active schedule block for each student.
///
/// Original reference: `StudentScript.Update` schedule/phase branch. A student in
/// an alarm state keeps its timetable phase but ignores the destination.
pub fn schedule_system(world: &mut World, minute: u16, events: &mut EventQueue) {
    for (_, (student, schedule)) in world.query_mut::<(&mut Student, &Schedule)>() {
        if !student.alive {
            continue;
        }
        let Some(block) = schedule.block_at(minute) else {
            continue;
        };
        if student.state.follows_schedule() {
            student.destination = block.destination;
        }
        if student.phase != block.phase {
            student.phase = block.phase;
            events.push(GameEvent::PhaseChanged {
                student_id: student.student_id,
                phase: block.phase,
            });
        }
    }
}

/// Computes visibility of the player from a student's position and facing.
///
/// Returns a factor in `0.0..=1.0`: zero when out of range or outside the vision
/// cone, fading toward the edge of the cone and the edge of the range so a
/// peripheral glimpse builds suspicion more slowly than a direct look.
pub fn visibility(observer: Vec3, facing: Vec3, target: Vec3, perception: &Perception) -> f32 {
    let offset = target - observer;
    let distance = offset.length();
    if distance > perception.range || distance <= f32::EPSILON {
        return 0.0;
    }
    let direction = offset / distance;
    let alignment = facing.normalize_or_zero().dot(direction);
    if alignment < perception.fov_cos {
        return 0.0;
    }

    // Normalize alignment from the cone edge (0) to dead center (1).
    let cone_span = (1.0 - perception.fov_cos).max(f32::EPSILON);
    let cone_factor = ((alignment - perception.fov_cos) / cone_span).clamp(0.0, 1.0);
    let range_factor = 1.0 - (distance / perception.range).clamp(0.0, 1.0);
    // Keep a floor so a valid sighting always registers something.
    (cone_factor * 0.5 + range_factor * 0.5).max(0.05)
}

/// Emits sighting events and records witness memories.
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
        if !student.alive || !student.state.is_aware() {
            continue;
        }
        if visibility(position.0, facing.0, player_position, perception) > 0.0 {
            events.push(GameEvent::StudentNoticedPlayer {
                student_id: student.student_id,
            });
        }
    }
}

/// Integrates suspicion from this tick's visibility and decays witness memory.
///
/// Original reference: suspicion accumulation scattered across `StudentScript`
/// branches. Suspicion is a component so it can be tuned per student.
pub fn suspicion_system(world: &mut World, clock: &SchoolClock, dt: f32) {
    let minute = clock.absolute_minute();
    let player_position = world
        .query::<(&Position, &Player)>()
        .iter()
        .next()
        .map(|(_, (position, _))| position.0);

    for (_, (position, facing, perception, student, suspicion, memory)) in world.query_mut::<(
        &Position,
        &Facing,
        &Perception,
        &Student,
        &mut Suspicion,
        &mut WitnessMemory,
    )>() {
        memory.advance_to(minute);
        if !student.alive || !student.state.is_aware() {
            continue;
        }

        let factor = match player_position {
            Some(target) => visibility(position.0, facing.0, target, perception),
            None => 0.0,
        };
        suspicion.advance(factor > 0.0, factor, dt);
    }
}

/// How far a fleeing student tries to put between itself and the threat.
pub const FLEE_DISTANCE: f32 = 25.0;

/// Resolves the behavior state machine and emits transitions.
///
/// On entering `Fleeing` the student picks an escape destination away from the
/// player. Without this, an alarm state would keep the timetable destination and
/// a fleeing student could run toward the thing it is fleeing from.
pub fn behavior_system(world: &mut World, events: &mut EventQueue) {
    let threat = world
        .query::<(&Position, &Player)>()
        .iter()
        .next()
        .map(|(_, (position, _))| position.0);

    for (_, (position, student, suspicion, schedule)) in
        world.query_mut::<(&Position, &mut Student, &Suspicion, &Schedule)>()
    {
        let distance = (student.destination - position.0).length();
        let next = behavior::next_state(TransitionInput {
            state: student.state,
            suspicion: suspicion.level,
            thresholds: suspicion.thresholds,
            at_destination: distance <= ARRIVAL_EPSILON,
            has_schedule_block: !schedule.blocks.is_empty(),
            alive: student.alive,
        });
        if next == student.state {
            continue;
        }
        if next == BehaviorState::Fleeing {
            // Escape along the horizontal plane away from the threat. With no
            // threat in the scene, fall back to the student's facing direction
            // encoded as +Z so the choice stays deterministic.
            let away = match threat {
                Some(source) => {
                    let offset = Vec3::new(position.0.x - source.x, 0.0, position.0.z - source.z);
                    if offset.length_squared() > f32::EPSILON {
                        offset.normalize()
                    } else {
                        Vec3::Z
                    }
                }
                None => Vec3::Z,
            };
            student.destination = position.0 + away * FLEE_DISTANCE;
        }
        student.state = next;
        events.push(GameEvent::BehaviorChanged {
            student_id: student.student_id,
            state: next,
        });
    }
}

/// Records and reports incidents for students that reached the report threshold.
///
/// The memory is cleared on report so one incident produces one report instead
/// of one per tick.
pub fn report_system(world: &mut World, events: &mut EventQueue) {
    for (_, (student, suspicion, memory)) in
        world.query_mut::<(&Student, &mut Suspicion, &mut WitnessMemory)>()
    {
        if !student.alive || !suspicion.should_report() {
            continue;
        }
        // A report needs something to report. Suspicion alone makes a student
        // flee, but staff are only involved when there is a witnessed cause.
        let Some(strongest) = memory.strongest().copied() else {
            continue;
        };
        events.push(GameEvent::IncidentReported {
            student_id: student.student_id,
            kind: strongest.kind,
        });
        memory.clear();
        suspicion.reset();
    }
}

/// Moves students toward their destination at the state's speed scale.
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
        // Snap before consulting the speed scale: behavior_system already
        // decided the student arrived, and a zero scale must not freeze it
        // short of the destination.
        if distance <= ARRIVAL_EPSILON {
            position.0 = student.destination;
            continue;
        }
        let scale = student.state.speed_scale();
        if scale <= 0.0 {
            continue;
        }
        let step = student.speed * scale * dt;
        if step >= distance {
            position.0 = student.destination;
        } else {
            position.0 += to_target / distance * step;
        }
    }
}

/// Records a witnessed event, used by gameplay systems and tests.
pub fn observe_incident(
    world: &mut World,
    student_id: crate::StudentId,
    kind: WitnessKind,
    subject: Option<crate::StudentId>,
    clock: &SchoolClock,
    confidence: f32,
) {
    for (_, (student, memory)) in world.query_mut::<(&Student, &mut WitnessMemory)>() {
        if student.student_id == student_id {
            memory.observe(kind, subject, clock.absolute_minute(), confidence);
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BehaviorState;
    use crate::{ScheduleBlock, SimConfig, Simulation, MINUTES_PER_DAY};

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

    fn perception() -> Perception {
        Perception {
            range: 10.0,
            fov_cos: (60.0_f32.to_radians()).cos(),
        }
    }

    #[test]
    fn visibility_is_zero_outside_the_cone() {
        assert_eq!(
            visibility(
                Vec3::ZERO,
                Vec3::Z,
                Vec3::new(0.0, 0.0, -5.0),
                &perception()
            ),
            0.0
        );
    }

    #[test]
    fn visibility_is_zero_out_of_range() {
        assert_eq!(
            visibility(
                Vec3::ZERO,
                Vec3::Z,
                Vec3::new(0.0, 0.0, 50.0),
                &perception()
            ),
            0.0
        );
    }

    #[test]
    fn closer_targets_are_more_visible() {
        let near = visibility(Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 1.0), &perception());
        let far = visibility(Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 9.0), &perception());
        assert!(near > far);
        assert!(near <= 1.0);
    }

    #[test]
    fn movement_snaps_on_arrival_and_stays() {
        let mut sim = Simulation::new(SimConfig::default());
        let entity = sim
            .spawn_student(1, Vec3::ZERO, schedule_to(Vec3::new(0.1, 0.0, 0.0), 1))
            .unwrap();
        for _ in 0..120 {
            sim.tick(1.0 / 60.0);
        }
        assert_eq!(
            sim.world.get::<&Position>(entity).unwrap().0,
            Vec3::new(0.1, 0.0, 0.0)
        );
        assert_eq!(
            sim.world.get::<&Student>(entity).unwrap().state,
            BehaviorState::Attending
        );
    }

    #[test]
    fn dead_students_do_not_move_or_emit_phase_events() {
        let mut sim = Simulation::new(SimConfig::default());
        let entity = sim
            .spawn_student(1, Vec3::ZERO, schedule_to(Vec3::X * 50.0, 7))
            .unwrap();
        sim.world.get::<&mut Student>(entity).unwrap().alive = false;
        for _ in 0..30 {
            sim.tick(1.0 / 60.0);
        }
        assert_eq!(sim.world.get::<&Position>(entity).unwrap().0, Vec3::ZERO);
        assert_eq!(
            sim.world.get::<&Student>(entity).unwrap().state,
            BehaviorState::Incapacitated
        );
    }

    #[test]
    fn perception_without_a_player_is_a_no_op() {
        let mut world = World::new();
        let mut events = EventQueue::new();
        perception_system(&mut world, &mut events);
        assert_eq!(events.current_len(), 0);
    }

    #[test]
    fn sustained_sighting_escalates_to_fleeing() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_player(Vec3::new(0.0, 0.0, 2.0));
        let entity = sim
            .spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        let mut states = Vec::new();
        for _ in 0..600 {
            sim.tick(1.0 / 60.0);
            let state = sim.world.get::<&Student>(entity).unwrap().state;
            if states.last() != Some(&state) {
                states.push(state);
            }
        }
        assert!(states.contains(&BehaviorState::Investigating));
        assert!(states.contains(&BehaviorState::Fleeing));
    }

    #[test]
    fn suspicion_decays_after_the_player_leaves() {
        let mut sim = Simulation::new(SimConfig::default());
        let player = sim.spawn_player(Vec3::new(0.0, 0.0, 2.0));
        let entity = sim
            .spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        for _ in 0..60 {
            sim.tick(1.0 / 60.0);
        }
        let peak = sim.world.get::<&Suspicion>(entity).unwrap().level;
        assert!(peak > 0.0);

        sim.world.get::<&mut Position>(player).unwrap().0 = Vec3::new(0.0, 0.0, 500.0);
        for _ in 0..600 {
            sim.tick(1.0 / 60.0);
        }
        assert_eq!(sim.world.get::<&Suspicion>(entity).unwrap().level, 0.0);
    }

    #[test]
    fn report_requires_a_witnessed_cause() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_player(Vec3::new(0.0, 0.0, 2.0));
        let entity = sim
            .spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        // Drive suspicion to the cap without any witnessed incident.
        sim.world.get::<&mut Suspicion>(entity).unwrap().level = 1.0;
        sim.tick(1.0 / 60.0);
        assert!(!sim
            .events
            .previous()
            .iter()
            .any(|event| matches!(event, GameEvent::IncidentReported { .. })));
    }

    #[test]
    fn witnessed_incident_is_reported_once() {
        let mut sim = Simulation::new(SimConfig::default());
        let entity = sim
            .spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        observe_incident(
            &mut sim.world,
            1,
            WitnessKind::Violence,
            Some(2),
            &sim.clock,
            1.0,
        );
        sim.world.get::<&mut Suspicion>(entity).unwrap().level = 1.0;

        sim.tick(1.0 / 60.0);
        let reports = sim
            .events
            .previous()
            .iter()
            .filter(|event| matches!(event, GameEvent::IncidentReported { .. }))
            .count();
        assert_eq!(reports, 1);

        // Suspicion was reset, so the next tick must not report again.
        sim.tick(1.0 / 60.0);
        assert!(!sim
            .events
            .previous()
            .iter()
            .any(|event| matches!(event, GameEvent::IncidentReported { .. })));
    }

    #[test]
    fn alarm_state_ignores_the_timetable_destination() {
        let mut sim = Simulation::new(SimConfig::default());
        let entity = sim
            .spawn_student(1, Vec3::ZERO, schedule_to(Vec3::X * 20.0, 1))
            .unwrap();
        {
            let mut student = sim.world.get::<&mut Student>(entity).unwrap();
            student.state = BehaviorState::Fleeing;
        }
        sim.world.get::<&mut Suspicion>(entity).unwrap().level = 0.8;
        let before = sim.world.get::<&Student>(entity).unwrap().destination;
        sim.tick(1.0 / 60.0);
        assert_eq!(
            sim.world.get::<&Student>(entity).unwrap().destination,
            before
        );
    }

    #[test]
    fn fleeing_moves_faster_than_walking() {
        let mut walker = Simulation::new(SimConfig::default());
        let mut fleer = Simulation::new(SimConfig::default());
        let target = Vec3::X * 100.0;
        let w = walker
            .spawn_student(1, Vec3::ZERO, schedule_to(target, 1))
            .unwrap();
        let f = fleer
            .spawn_student(1, Vec3::ZERO, schedule_to(target, 1))
            .unwrap();
        // Threat behind the student, so escaping means running along +X.
        fleer.spawn_player(Vec3::X * -5.0);
        fleer.world.get::<&mut Suspicion>(f).unwrap().level = 0.8;

        for _ in 0..60 {
            walker.tick(1.0 / 60.0);
            // Hold suspicion at the flee band; with no player in this scene it
            // would otherwise decay into investigating and slow down.
            fleer.world.get::<&mut Suspicion>(f).unwrap().level = 0.8;
            fleer.tick(1.0 / 60.0);
        }
        let walked = walker.world.get::<&Position>(w).unwrap().0.length();
        let fled = fleer.world.get::<&Position>(f).unwrap().0.length();
        assert!(fled > walked);
    }
}

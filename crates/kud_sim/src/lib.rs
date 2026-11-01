//! Data-oriented game simulation.
//!
//! The simulation is independent of Unity, rendering, and windowing. It uses
//! stable student IDs, explicit schedules, and deterministic event ordering.
//! Original C# identifiers are preserved in the porting map and serialized
//! field names where compatibility is required; Rust code uses idiomatic names.

pub mod events;
pub mod input;
pub mod registry;
pub mod replay;
pub mod scenario;
pub mod systems;

use glam::{Quat, Vec3};
use kud_core::StateHasher;
use kud_ecs::{Entity, World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use events::EventQueue;
pub use input::InputCommand;
pub use registry::StudentRegistry;
pub use replay::{Replay, ReplayError};

/// Original reference: `StudentManagerScript.Students` index.
pub type StudentId = u32;

/// Errors produced by the simulation layer.
#[derive(Debug, Error)]
pub enum SimError {
    /// A student ID is already bound to an entity.
    #[error("student id {0} is already registered")]
    DuplicateStudentId(StudentId),

    /// A student ID is reserved and cannot be assigned.
    #[error("student id 0 is reserved")]
    ReservedStudentId,

    /// No student is registered under the requested ID.
    #[error("unknown student id {0}")]
    UnknownStudentId(StudentId),
}

/// World-space position. Original reference: `Transform.position`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position(pub Vec3);

/// World-space rotation. Original reference: `Transform.rotation`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation(pub Quat);

/// Marker for the player character. Original reference: `YandereScript`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Player;

/// Student runtime state. Original reference: `StudentScript`.
#[derive(Debug, Clone, PartialEq)]
pub struct Student {
    /// Original field: `StudentID`.
    pub student_id: StudentId,
    /// Original field: `Phase`.
    pub phase: u8,
    /// Original field: `Destination` / `CurrentDestination`.
    pub destination: Vec3,
    /// Movement speed in meters per second.
    pub speed: f32,
    /// Original field: `Alive`.
    pub alive: bool,
}

/// One scheduled activity. Original reference: `StudentScript.ScheduleBlock`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleBlock {
    /// Original field: `time`.
    pub start_minute: u16,
    /// Exclusive end minute.
    pub end_minute: u16,
    /// Original field: `destination`.
    pub destination: Vec3,
    /// Original field: `action`.
    pub action: String,
    /// Original field: `phase`.
    pub phase: u8,
}

impl ScheduleBlock {
    /// Returns true when the block covers the given minute.
    #[inline]
    pub fn contains_minute(&self, minute: u16) -> bool {
        minute >= self.start_minute && minute < self.end_minute
    }
}

/// A student's daily schedule.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    /// Ordered schedule blocks.
    pub blocks: Vec<ScheduleBlock>,
}

impl Schedule {
    /// Returns the block covering a minute, if any.
    ///
    /// The first match wins so overlapping blocks resolve deterministically.
    pub fn block_at(&self, minute: u16) -> Option<&ScheduleBlock> {
        self.blocks
            .iter()
            .find(|block| block.contains_minute(minute))
    }
}

/// Simple vision model. Original reference: student vision/detection fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Perception {
    /// Maximum sight distance in meters.
    pub range: f32,
    /// Cosine of the half field-of-view angle.
    pub fov_cos: f32,
}

/// Forward direction used by perception.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Facing(pub Vec3);

/// School clock. Original reference: `ClockScript` / `PresentTime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolClock {
    /// Minutes since midnight.
    pub minute: u16,
    /// Day index.
    pub day: u16,
}

/// Minutes in a full day.
pub const MINUTES_PER_DAY: u16 = 24 * 60;

impl Default for SchoolClock {
    fn default() -> Self {
        Self {
            minute: 7 * 60,
            day: 0,
        }
    }
}

impl SchoolClock {
    /// Advances the clock, rolling over into the next day.
    pub fn advance(&mut self, minutes: u16) {
        let total = u32::from(self.minute) + u32::from(minutes);
        self.minute = (total % u32::from(MINUTES_PER_DAY)) as u16;
        self.day = self
            .day
            .saturating_add((total / u32::from(MINUTES_PER_DAY)) as u16);
    }
}

/// Deterministic simulation events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    /// A student changed schedule phase.
    PhaseChanged {
        /// Student that changed phase.
        student_id: StudentId,
        /// New phase.
        phase: u8,
    },
    /// A student saw the player.
    StudentNoticedPlayer {
        /// Student that noticed the player.
        student_id: StudentId,
    },
}

/// Simulation configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SimConfig {
    /// In-game minutes advanced per fixed tick.
    pub minutes_per_tick: u16,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            minutes_per_tick: 1,
        }
    }
}

/// Owns the ECS world and simulation resources.
pub struct Simulation {
    /// ECS world.
    pub world: World,
    /// Stable student ID to entity map.
    pub registry: StudentRegistry,
    /// School clock.
    pub clock: SchoolClock,
    /// Simulation configuration.
    pub config: SimConfig,
    /// Double-buffered event queue.
    pub events: EventQueue,
    /// Total fixed ticks executed.
    pub tick_count: u64,
}

impl Simulation {
    /// Creates an empty simulation.
    pub fn new(config: SimConfig) -> Self {
        Self {
            world: World::new(),
            registry: StudentRegistry::new(),
            clock: SchoolClock::default(),
            config,
            events: EventQueue::new(),
            tick_count: 0,
        }
    }

    /// Spawns the player at a position.
    pub fn spawn_player(&mut self, position: Vec3) -> Entity {
        self.world
            .spawn((Position(position), Rotation(Quat::IDENTITY), Player))
    }

    /// Spawns a student with a schedule and registers its stable ID.
    ///
    /// Rejects duplicate IDs instead of silently overwriting a binding, which
    /// would orphan the previous entity.
    pub fn spawn_student(
        &mut self,
        student_id: StudentId,
        position: Vec3,
        schedule: Schedule,
    ) -> Result<Entity, SimError> {
        if student_id == 0 {
            return Err(SimError::ReservedStudentId);
        }
        if self.registry.contains(student_id) {
            return Err(SimError::DuplicateStudentId(student_id));
        }
        let entity = self.world.spawn((
            Position(position),
            Rotation(Quat::IDENTITY),
            Facing(Vec3::Z),
            Perception {
                range: 10.0,
                fov_cos: (60.0_f32.to_radians()).cos(),
            },
            Student {
                student_id,
                phase: 0,
                destination: position,
                speed: 1.4,
                alive: true,
            },
            schedule,
        ));
        self.registry.insert(student_id, entity);
        Ok(entity)
    }

    /// Despawns a student and releases its stable ID.
    pub fn despawn_student(&mut self, student_id: StudentId) -> Result<(), SimError> {
        let entity = self
            .registry
            .remove(student_id)
            .ok_or(SimError::UnknownStudentId(student_id))?;
        let _ = self.world.despawn(entity);
        Ok(())
    }

    /// Returns the entity bound to a student ID.
    pub fn student_entity(&self, student_id: StudentId) -> Result<Entity, SimError> {
        self.registry
            .entity(student_id)
            .ok_or(SimError::UnknownStudentId(student_id))
    }

    /// Advances one fixed simulation tick.
    ///
    /// Systems run in a fixed order. Events written during this tick become
    /// readable through `events.previous()` after the queue flips.
    pub fn tick(&mut self, dt: f32) {
        self.tick_with_input(dt, InputCommand::Idle);
    }

    /// Advances one fixed tick after applying an input command.
    pub fn tick_with_input(&mut self, dt: f32, command: InputCommand) {
        self.clock.advance(self.config.minutes_per_tick);

        input::input_system(&mut self.world, command, dt);
        systems::schedule_system(&mut self.world, self.clock.minute, &mut self.events);
        systems::movement_system(&mut self.world, dt);
        systems::perception_system(&mut self.world, &mut self.events);

        self.events.flip();
        self.tick_count += 1;
    }

    /// Computes a deterministic hash of the observable simulation state.
    ///
    /// Students are visited in ascending `StudentId` order, not ECS order, so
    /// the hash is independent of spawn order and archetype layout. Transient
    /// buffers and the event queue are excluded: they are outputs of a tick,
    /// not state that carries forward.
    pub fn state_hash(&self) -> u64 {
        let mut hasher = StateHasher::new();
        hasher.write_u64(self.tick_count);
        hasher.write_u16(self.clock.minute);
        hasher.write_u16(self.clock.day);
        hasher.write_u32(self.registry.len() as u32);

        for (student_id, entity) in self.registry.iter() {
            hasher.write_u32(student_id);
            let Ok(student) = self.world.get::<&Student>(entity) else {
                continue;
            };
            hasher.write_u8(student.phase);
            hasher.write_bool(student.alive);
            hasher.write_f32(student.speed);
            hasher.write_vec3(student.destination);
            if let Ok(position) = self.world.get::<&Position>(entity) {
                hasher.write_vec3(position.0);
            }
        }

        if let Some((_, (position, _))) = self.world.query::<(&Position, &Player)>().iter().next() {
            hasher.write_bool(true);
            hasher.write_vec3(position.0);
        } else {
            hasher.write_bool(false);
        }

        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_schedule() -> Schedule {
        Schedule {
            blocks: vec![
                ScheduleBlock {
                    start_minute: 420,
                    end_minute: 480,
                    destination: Vec3::new(0.0, 0.0, 0.0),
                    action: "Locker".to_string(),
                    phase: 1,
                },
                ScheduleBlock {
                    start_minute: 480,
                    end_minute: 540,
                    destination: Vec3::new(10.0, 0.0, 0.0),
                    action: "Class".to_string(),
                    phase: 2,
                },
            ],
        }
    }

    fn walking_schedule() -> Schedule {
        Schedule {
            blocks: vec![ScheduleBlock {
                start_minute: 0,
                end_minute: MINUTES_PER_DAY,
                destination: Vec3::new(10.0, 0.0, 0.0),
                action: "Walk".to_string(),
                phase: 1,
            }],
        }
    }

    #[test]
    fn schedule_changes_phase_and_destination() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_student(1, Vec3::ZERO, sample_schedule()).unwrap();
        sim.clock.minute = 479;
        sim.tick(1.0 / 60.0);
        assert!(sim.events.previous().contains(&GameEvent::PhaseChanged {
            student_id: 1,
            phase: 2,
        }));
    }

    #[test]
    fn movement_is_deterministic() {
        let mut a = Simulation::new(SimConfig::default());
        let mut b = Simulation::new(SimConfig::default());
        a.spawn_student(1, Vec3::ZERO, walking_schedule()).unwrap();
        b.spawn_student(1, Vec3::ZERO, walking_schedule()).unwrap();
        for _ in 0..120 {
            a.tick(1.0 / 60.0);
            b.tick(1.0 / 60.0);
        }
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn perception_emits_event_inside_fov() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_player(Vec3::new(0.0, 0.0, 5.0));
        sim.spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        sim.tick(1.0 / 60.0);
        assert!(sim
            .events
            .previous()
            .contains(&GameEvent::StudentNoticedPlayer { student_id: 1 }));
    }

    #[test]
    fn perception_ignores_player_behind_the_student() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_player(Vec3::new(0.0, 0.0, -5.0));
        sim.spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        sim.tick(1.0 / 60.0);
        assert!(sim.events.previous().is_empty());
    }

    #[test]
    fn duplicate_student_id_is_rejected() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        assert!(matches!(
            sim.spawn_student(1, Vec3::ZERO, Schedule::default()),
            Err(SimError::DuplicateStudentId(1))
        ));
        assert_eq!(sim.registry.len(), 1);
    }

    #[test]
    fn student_id_zero_is_reserved() {
        let mut sim = Simulation::new(SimConfig::default());
        assert!(matches!(
            sim.spawn_student(0, Vec3::ZERO, Schedule::default()),
            Err(SimError::ReservedStudentId)
        ));
    }

    #[test]
    fn despawn_releases_the_id() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        sim.despawn_student(1).unwrap();
        assert!(!sim.registry.contains(1));
        assert!(matches!(
            sim.despawn_student(1),
            Err(SimError::UnknownStudentId(1))
        ));
        sim.spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
    }

    #[test]
    fn state_hash_ignores_spawn_order() {
        let mut a = Simulation::new(SimConfig::default());
        let mut b = Simulation::new(SimConfig::default());
        for id in [1_u32, 2, 3] {
            a.spawn_student(id, Vec3::ZERO, walking_schedule()).unwrap();
        }
        for id in [3_u32, 1, 2] {
            b.spawn_student(id, Vec3::ZERO, walking_schedule()).unwrap();
        }
        for _ in 0..30 {
            a.tick(1.0 / 60.0);
            b.tick(1.0 / 60.0);
        }
        assert_eq!(a.state_hash(), b.state_hash());
    }

    #[test]
    fn state_hash_detects_position_change() {
        let mut sim = Simulation::new(SimConfig::default());
        let entity = sim
            .spawn_student(1, Vec3::ZERO, Schedule::default())
            .unwrap();
        let before = sim.state_hash();
        sim.world.get::<&mut Position>(entity).unwrap().0 = Vec3::X;
        assert_ne!(sim.state_hash(), before);
    }

    #[test]
    fn school_clock_rolls_over_into_the_next_day() {
        let mut clock = SchoolClock {
            minute: MINUTES_PER_DAY - 1,
            day: 0,
        };
        clock.advance(2);
        assert_eq!(clock.minute, 1);
        assert_eq!(clock.day, 1);
    }

    #[test]
    fn schedule_block_lookup_uses_exclusive_end() {
        let schedule = sample_schedule();
        assert_eq!(schedule.block_at(479).unwrap().phase, 1);
        assert_eq!(schedule.block_at(480).unwrap().phase, 2);
        assert!(schedule.block_at(540).is_none());
    }
}

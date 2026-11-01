//! Data-oriented game simulation.
//!
//! The simulation is independent of Unity, rendering, and windowing. It uses
//! stable student IDs, explicit schedules, and deterministic event ordering.
//! Original C# identifiers are preserved in the porting map and serialized
//! field names where compatibility is required; Rust code uses idiomatic names.

use glam::{Quat, Vec3};
use kud_ecs::{Entity, World};
use serde::{Deserialize, Serialize};

/// Original reference: `StudentManagerScript.Students` index.
pub type StudentId = u32;

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

/// A student's daily schedule.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    /// Ordered schedule blocks.
    pub blocks: Vec<ScheduleBlock>,
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

impl Default for SchoolClock {
    fn default() -> Self {
        Self {
            minute: 7 * 60,
            day: 0,
        }
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
#[derive(Debug, Clone, Copy)]
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
    /// School clock.
    pub clock: SchoolClock,
    /// Simulation configuration.
    pub config: SimConfig,
    /// Events produced by the current tick.
    pub events: Vec<GameEvent>,
    /// Total fixed ticks executed.
    pub tick_count: u64,
}

impl Simulation {
    /// Creates an empty simulation.
    pub fn new(config: SimConfig) -> Self {
        Self {
            world: World::new(),
            clock: SchoolClock::default(),
            config,
            events: Vec::new(),
            tick_count: 0,
        }
    }

    /// Spawns the player at a position.
    pub fn spawn_player(&mut self, position: Vec3) -> Entity {
        self.world
            .spawn((Position(position), Rotation(Quat::IDENTITY), Player))
    }

    /// Spawns a student with a schedule.
    pub fn spawn_student(
        &mut self,
        student_id: StudentId,
        position: Vec3,
        schedule: Schedule,
    ) -> Entity {
        self.world.spawn((
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
        ))
    }

    /// Advances one fixed simulation tick.
    pub fn tick(&mut self, dt: f32) {
        self.events.clear();
        self.clock.minute = self
            .clock
            .minute
            .saturating_add(self.config.minutes_per_tick);
        if self.clock.minute >= 24 * 60 {
            self.clock.minute -= 24 * 60;
            self.clock.day = self.clock.day.saturating_add(1);
        }

        schedule_system(&mut self.world, self.clock.minute, &mut self.events);
        movement_system(&mut self.world, dt);
        perception_system(&mut self.world, &mut self.events);

        // Stable ordering makes replay and test output independent of archetype layout.
        self.events.sort_by_key(|event| match event {
            GameEvent::PhaseChanged { student_id, phase } => (*student_id, 0, *phase),
            GameEvent::StudentNoticedPlayer { student_id } => (*student_id, 1, 0),
        });
        self.tick_count += 1;
    }
}

/// Selects the active schedule block for each student.
pub fn schedule_system(world: &mut World, minute: u16, events: &mut Vec<GameEvent>) {
    for (_, (student, schedule)) in world.query_mut::<(&mut Student, &Schedule)>() {
        if !student.alive {
            continue;
        }
        if let Some(block) = schedule
            .blocks
            .iter()
            .find(|block| minute >= block.start_minute && minute < block.end_minute)
        {
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
}

/// Moves students toward their current destination.
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
pub fn perception_system(world: &mut World, events: &mut Vec<GameEvent>) {
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

    #[test]
    fn schedule_changes_phase_and_destination() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_student(1, Vec3::ZERO, sample_schedule());
        sim.clock.minute = 479;
        sim.tick(1.0 / 60.0);
        assert!(sim.events.contains(&GameEvent::PhaseChanged {
            student_id: 1,
            phase: 2,
        }));
    }

    #[test]
    fn movement_is_deterministic() {
        let mut a = Simulation::new(SimConfig::default());
        let mut b = Simulation::new(SimConfig::default());
        let schedule = Schedule {
            blocks: vec![ScheduleBlock {
                start_minute: 0,
                end_minute: 1440,
                destination: Vec3::new(10.0, 0.0, 0.0),
                action: "Walk".to_string(),
                phase: 1,
            }],
        };
        a.spawn_student(1, Vec3::ZERO, schedule.clone());
        b.spawn_student(1, Vec3::ZERO, schedule);
        for _ in 0..120 {
            a.tick(1.0 / 60.0);
            b.tick(1.0 / 60.0);
        }
        let pa = a.world.query::<&Position>().iter().next().unwrap().1 .0;
        let pb = b.world.query::<&Position>().iter().next().unwrap().1 .0;
        assert_eq!(pa, pb);
    }

    #[test]
    fn perception_emits_event_inside_fov() {
        let mut sim = Simulation::new(SimConfig::default());
        sim.spawn_player(Vec3::new(0.0, 0.0, 5.0));
        sim.spawn_student(1, Vec3::ZERO, Schedule::default());
        sim.tick(1.0 / 60.0);
        assert!(sim
            .events
            .contains(&GameEvent::StudentNoticedPlayer { student_id: 1 }));
    }
}

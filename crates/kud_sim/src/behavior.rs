//! Student behavior state machine.
//!
//! Original reference: the branch cascade in `StudentScript.Update`, which mixed
//! schedule, movement, social, and alarm behavior in one method. Here the state
//! is explicit data and transitions are a pure function, so a transition table
//! can be tested without running a frame.
//!
//! The schedule phase and the behavior state are deliberately separate: phase is
//! *where the timetable says to be*, state is *what the student is doing about
//! it*. A student can be in the class phase and still be fleeing.

use serde::{Deserialize, Serialize};

use crate::StudentId;

/// What a student is currently doing.
///
/// Original reference: `StudentScript` behavior flags (`Panic`, `GoingToClass`,
/// `Socializing`, and similar), consolidated into one exhaustive enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BehaviorState {
    /// Standing still with nothing to do.
    #[default]
    Idle,
    /// Moving toward the current schedule destination.
    Walking,
    /// Arrived and performing the scheduled activity.
    Attending,
    /// Talking with another student.
    Socializing,
    /// Suspicious enough to look at or approach the player.
    Investigating,
    /// Running away from a threat.
    Fleeing,
    /// On the way to report an incident to staff.
    Reporting,
    /// Restrained or otherwise unable to act.
    Incapacitated,
}

impl BehaviorState {
    /// Returns true when the state allows schedule-driven movement.
    pub fn follows_schedule(self) -> bool {
        matches!(self, Self::Idle | Self::Walking | Self::Attending)
    }

    /// Returns true when the student can perceive and react to the world.
    pub fn is_aware(self) -> bool {
        !matches!(self, Self::Incapacitated)
    }

    /// Returns true when the state overrides the timetable.
    pub fn is_alarm(self) -> bool {
        matches!(self, Self::Investigating | Self::Fleeing | Self::Reporting)
    }

    /// Movement speed multiplier applied by this state.
    pub fn speed_scale(self) -> f32 {
        match self {
            Self::Fleeing => 2.0,
            Self::Reporting => 1.6,
            Self::Investigating => 0.6,
            Self::Idle | Self::Attending | Self::Socializing | Self::Incapacitated => 0.0,
            Self::Walking => 1.0,
        }
    }
}

/// Suspicion thresholds that drive alarm transitions.
///
/// Exposed as data so designers can tune them per student without touching code.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SuspicionThresholds {
    /// Suspicion at which the student starts investigating.
    pub investigate: f32,
    /// Suspicion at which the student flees.
    pub flee: f32,
    /// Suspicion at which the student reports to staff.
    pub report: f32,
}

impl Default for SuspicionThresholds {
    fn default() -> Self {
        Self {
            investigate: 0.35,
            flee: 0.7,
            report: 1.0,
        }
    }
}

impl SuspicionThresholds {
    /// Returns true when the thresholds are strictly increasing and finite.
    pub fn is_valid(&self) -> bool {
        self.investigate.is_finite()
            && self.flee.is_finite()
            && self.report.is_finite()
            && 0.0 < self.investigate
            && self.investigate < self.flee
            && self.flee < self.report
    }
}

/// Per-student awareness of the player.
///
/// Original reference: `StudentScript` suspicion accumulation, which was spread
/// across several `Update` branches with per-frame constants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Suspicion {
    /// Current suspicion in `0.0..=report` threshold.
    pub level: f32,
    /// Suspicion gained per second while the player is visible.
    pub gain_per_second: f32,
    /// Suspicion lost per second while the player is not visible.
    pub decay_per_second: f32,
    /// Thresholds that trigger state changes.
    pub thresholds: SuspicionThresholds,
}

impl Default for Suspicion {
    fn default() -> Self {
        Self {
            level: 0.0,
            gain_per_second: 0.5,
            decay_per_second: 0.25,
            thresholds: SuspicionThresholds::default(),
        }
    }
}

impl Suspicion {
    /// Advances suspicion by one tick.
    ///
    /// `visibility` scales the gain so a distant or peripheral sighting builds
    /// suspicion more slowly than a direct one. It is clamped to `0.0..=1.0`.
    pub fn advance(&mut self, visible: bool, visibility: f32, dt: f32) {
        if visible {
            self.level += self.gain_per_second * visibility.clamp(0.0, 1.0) * dt;
        } else {
            self.level -= self.decay_per_second * dt;
        }
        self.level = self.level.clamp(0.0, self.thresholds.report);
    }

    /// Returns true when suspicion reached the reporting threshold.
    pub fn should_report(&self) -> bool {
        self.level >= self.thresholds.report
    }

    /// Resets suspicion to zero, used when an incident is resolved.
    pub fn reset(&mut self) {
        self.level = 0.0;
    }
}

/// Inputs to the behavior transition function.
#[derive(Debug, Clone, Copy)]
pub struct TransitionInput {
    /// Current behavior state.
    pub state: BehaviorState,
    /// Current suspicion.
    pub suspicion: f32,
    /// Thresholds for this student.
    pub thresholds: SuspicionThresholds,
    /// True when the student has arrived at its destination.
    pub at_destination: bool,
    /// True when a schedule block is active.
    pub has_schedule_block: bool,
    /// True when the student is live and able to act.
    pub alive: bool,
}

/// Computes the next behavior state.
///
/// Pure function: no ECS access, no events, no side effects. Alarm states take
/// precedence over the timetable, and de-escalation only happens once suspicion
/// falls below the investigate threshold, which prevents state flapping at a
/// boundary.
pub fn next_state(input: TransitionInput) -> BehaviorState {
    if !input.alive {
        return BehaviorState::Incapacitated;
    }

    if input.suspicion >= input.thresholds.report {
        return BehaviorState::Reporting;
    }
    if input.suspicion >= input.thresholds.flee {
        return BehaviorState::Fleeing;
    }
    if input.suspicion >= input.thresholds.investigate {
        return BehaviorState::Investigating;
    }

    // Below every threshold the student resumes its timetable. De-escalation is
    // handled by the ordered threshold checks above, which drop one band at a
    // time; an early return for alarm states here would latch them forever.
    if !input.has_schedule_block {
        return BehaviorState::Idle;
    }
    if input.at_destination {
        BehaviorState::Attending
    } else {
        BehaviorState::Walking
    }
}

/// A behavior transition emitted as an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateChange {
    /// Student whose state changed.
    pub student_id: StudentId,
    /// Previous state.
    pub from: BehaviorState,
    /// New state.
    pub to: BehaviorState,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: BehaviorState, suspicion: f32) -> TransitionInput {
        TransitionInput {
            state,
            suspicion,
            thresholds: SuspicionThresholds::default(),
            at_destination: false,
            has_schedule_block: true,
            alive: true,
        }
    }

    #[test]
    fn default_thresholds_are_valid() {
        assert!(SuspicionThresholds::default().is_valid());
    }

    #[test]
    fn unordered_thresholds_are_invalid() {
        let thresholds = SuspicionThresholds {
            investigate: 0.8,
            flee: 0.5,
            report: 1.0,
        };
        assert!(!thresholds.is_valid());
    }

    #[test]
    fn walks_when_far_and_attends_when_arrived() {
        assert_eq!(
            next_state(input(BehaviorState::Idle, 0.0)),
            BehaviorState::Walking
        );
        let mut arrived = input(BehaviorState::Walking, 0.0);
        arrived.at_destination = true;
        assert_eq!(next_state(arrived), BehaviorState::Attending);
    }

    #[test]
    fn no_schedule_block_means_idle() {
        let mut no_block = input(BehaviorState::Walking, 0.0);
        no_block.has_schedule_block = false;
        assert_eq!(next_state(no_block), BehaviorState::Idle);
    }

    #[test]
    fn suspicion_escalates_through_the_thresholds() {
        assert_eq!(
            next_state(input(BehaviorState::Walking, 0.4)),
            BehaviorState::Investigating
        );
        assert_eq!(
            next_state(input(BehaviorState::Walking, 0.8)),
            BehaviorState::Fleeing
        );
        assert_eq!(
            next_state(input(BehaviorState::Walking, 1.0)),
            BehaviorState::Reporting
        );
    }

    #[test]
    fn alarm_state_persists_until_suspicion_decays() {
        // Below flee but above investigate: de-escalate one band, never
        // straight back to the timetable.
        let fleeing = input(BehaviorState::Fleeing, 0.5);
        assert_eq!(next_state(fleeing), BehaviorState::Investigating);

        // Still above investigate: an alarm state must not resume walking.
        let uneasy = input(BehaviorState::Investigating, 0.36);
        assert_eq!(next_state(uneasy), BehaviorState::Investigating);

        let calm = input(BehaviorState::Fleeing, 0.1);
        assert_eq!(next_state(calm), BehaviorState::Walking);
    }

    #[test]
    fn dead_students_are_incapacitated_regardless_of_suspicion() {
        let mut dead = input(BehaviorState::Walking, 1.0);
        dead.alive = false;
        assert_eq!(next_state(dead), BehaviorState::Incapacitated);
    }

    #[test]
    fn suspicion_gain_respects_visibility_scale() {
        let mut full = Suspicion::default();
        let mut half = Suspicion::default();
        full.advance(true, 1.0, 1.0);
        half.advance(true, 0.5, 1.0);
        assert!(full.level > half.level);
        assert!((half.level - full.level / 2.0).abs() < 1e-6);
    }

    #[test]
    fn suspicion_decays_when_not_visible_and_floors_at_zero() {
        let mut suspicion = Suspicion::default();
        suspicion.advance(true, 1.0, 1.0);
        let peak = suspicion.level;
        suspicion.advance(false, 0.0, 1.0);
        assert!(suspicion.level < peak);
        suspicion.advance(false, 0.0, 100.0);
        assert_eq!(suspicion.level, 0.0);
    }

    #[test]
    fn suspicion_caps_at_the_report_threshold() {
        let mut suspicion = Suspicion::default();
        suspicion.advance(true, 1.0, 1000.0);
        assert_eq!(suspicion.level, suspicion.thresholds.report);
        assert!(suspicion.should_report());
    }

    #[test]
    fn visibility_is_clamped() {
        let mut over = Suspicion::default();
        let mut normal = Suspicion::default();
        over.advance(true, 5.0, 1.0);
        normal.advance(true, 1.0, 1.0);
        assert_eq!(over.level, normal.level);
    }

    #[test]
    fn alarm_states_move_faster_and_idle_states_do_not_move() {
        assert!(BehaviorState::Fleeing.speed_scale() > BehaviorState::Walking.speed_scale());
        assert_eq!(BehaviorState::Attending.speed_scale(), 0.0);
        assert_eq!(BehaviorState::Incapacitated.speed_scale(), 0.0);
    }

    #[test]
    fn only_schedule_states_follow_the_timetable() {
        assert!(BehaviorState::Walking.follows_schedule());
        assert!(!BehaviorState::Fleeing.follows_schedule());
        assert!(!BehaviorState::Reporting.follows_schedule());
    }
}

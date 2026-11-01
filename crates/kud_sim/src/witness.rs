//! Witness memory with confidence decay.
//!
//! Original reference: `StudentScript` witness flags, which were booleans that
//! never expired. A student who saw something remembered it forever at full
//! certainty, so stealth had no time dimension.
//!
//! Here each memory is a record with a confidence that decays, keyed by what was
//! seen. Repeated sightings reinforce the same memory instead of appending
//! duplicates, so the buffer stays bounded during a long incident.

use serde::{Deserialize, Serialize};

use crate::StudentId;

/// What a student witnessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessKind {
    /// The player was carrying something suspicious.
    SuspiciousItem,
    /// The player was somewhere they should not be.
    Trespassing,
    /// The player was covered in blood.
    Blood,
    /// The player attacked someone.
    Violence,
    /// A body was seen, without an identified perpetrator.
    Body,
}

impl WitnessKind {
    /// Base severity used to weight reports, in `0.0..=1.0`.
    pub fn severity(self) -> f32 {
        match self {
            Self::SuspiciousItem => 0.3,
            Self::Trespassing => 0.2,
            Self::Blood => 0.6,
            Self::Body => 0.8,
            Self::Violence => 1.0,
        }
    }

    /// Confidence lost per in-game minute.
    pub fn decay_per_minute(self) -> f32 {
        // Violent events are remembered far longer than a dress-code violation.
        match self {
            Self::Violence => 0.002,
            Self::Body => 0.004,
            Self::Blood => 0.01,
            Self::SuspiciousItem => 0.02,
            Self::Trespassing => 0.03,
        }
    }
}

/// One remembered observation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    /// What was observed.
    pub kind: WitnessKind,
    /// Who was observed, when identified.
    pub subject: Option<StudentId>,
    /// Current confidence in `0.0..=1.0`.
    pub confidence: f32,
    /// Absolute in-game minute of the most recent sighting.
    pub last_seen_minute: u32,
    /// Absolute minute decay has already been applied through.
    ///
    /// Separate from `last_seen_minute`: decay must be idempotent for a given
    /// minute, while a sighting must still be able to report when it happened.
    pub decayed_to_minute: u32,
    /// How many times this observation was reinforced.
    pub sightings: u32,
}

impl Memory {
    /// Creates a fresh memory at full confidence.
    pub fn new(
        kind: WitnessKind,
        subject: Option<StudentId>,
        minute: u32,
        initial_confidence: f32,
    ) -> Self {
        Self {
            kind,
            subject,
            confidence: initial_confidence.clamp(0.0, 1.0),
            last_seen_minute: minute,
            decayed_to_minute: minute,
            sightings: 1,
        }
    }

    /// Decays confidence to the given absolute minute.
    ///
    /// Uses absolute time rather than accumulating a per-tick delta, so the
    /// result does not depend on how many ticks elapsed.
    pub fn decay_to(&mut self, minute: u32) {
        let elapsed = minute.saturating_sub(self.decayed_to_minute);
        if elapsed == 0 {
            return;
        }
        self.confidence -= self.kind.decay_per_minute() * elapsed as f32;
        self.confidence = self.confidence.max(0.0);
        self.decayed_to_minute = minute;
    }

    /// Reinforces the memory from a new sighting.
    pub fn reinforce(&mut self, minute: u32, confidence: f32) {
        self.confidence = self.confidence.max(confidence.clamp(0.0, 1.0));
        self.last_seen_minute = self.last_seen_minute.max(minute);
        self.sightings = self.sightings.saturating_add(1);
    }

    /// Returns true when the memory is too weak to act on.
    pub fn is_forgotten(&self) -> bool {
        self.confidence <= f32::EPSILON
    }

    /// Report weight: confidence scaled by the severity of what was seen.
    pub fn weight(&self) -> f32 {
        self.confidence * self.kind.severity()
    }
}

/// Bounded set of memories for one student.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessMemory {
    /// Memories ordered by insertion, deduplicated by kind and subject.
    memories: Vec<Memory>,
    /// Confidence below which a memory is dropped.
    pub forget_threshold: f32,
    /// Maximum retained memories.
    pub capacity: usize,
}

impl Default for WitnessMemory {
    fn default() -> Self {
        Self {
            memories: Vec::new(),
            forget_threshold: 0.05,
            capacity: 16,
        }
    }
}

impl WitnessMemory {
    /// Records an observation, reinforcing an existing match instead of adding.
    pub fn observe(
        &mut self,
        kind: WitnessKind,
        subject: Option<StudentId>,
        minute: u32,
        confidence: f32,
    ) {
        if let Some(existing) = self
            .memories
            .iter_mut()
            .find(|memory| memory.kind == kind && memory.subject == subject)
        {
            existing.decay_to(minute);
            existing.reinforce(minute, confidence);
            return;
        }
        if self.memories.len() >= self.capacity {
            // Evict the weakest memory so a long incident cannot grow unbounded.
            if let Some(weakest) = self
                .memories
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.weight().total_cmp(&b.1.weight()))
                .map(|(index, _)| index)
            {
                self.memories.swap_remove(weakest);
            }
        }
        self.memories
            .push(Memory::new(kind, subject, minute, confidence));
    }

    /// Decays every memory and drops the forgotten ones.
    pub fn advance_to(&mut self, minute: u32) {
        for memory in &mut self.memories {
            memory.decay_to(minute);
        }
        self.memories
            .retain(|memory| memory.confidence > self.forget_threshold);
    }

    /// Returns retained memories.
    pub fn memories(&self) -> &[Memory] {
        &self.memories
    }

    /// Number of retained memories.
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// True when nothing is remembered.
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// Sum of report weights, used to decide whether to report to staff.
    pub fn total_weight(&self) -> f32 {
        self.memories.iter().map(Memory::weight).sum()
    }

    /// Strongest memory, if any.
    pub fn strongest(&self) -> Option<&Memory> {
        self.memories
            .iter()
            .max_by(|a, b| a.weight().total_cmp(&b.weight()))
    }

    /// Clears all memories, used when an incident is resolved.
    pub fn clear(&mut self) {
        self.memories.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_is_retained() {
        let mut memory = WitnessMemory::default();
        memory.observe(WitnessKind::Violence, Some(1), 0, 1.0);
        assert_eq!(memory.len(), 1);
        assert_eq!(memory.memories()[0].sightings, 1);
    }

    #[test]
    fn repeated_observation_reinforces_instead_of_duplicating() {
        let mut memory = WitnessMemory::default();
        memory.observe(WitnessKind::Blood, Some(1), 0, 0.5);
        memory.observe(WitnessKind::Blood, Some(1), 10, 0.9);
        assert_eq!(memory.len(), 1);
        assert_eq!(memory.memories()[0].sightings, 2);
        assert!(memory.memories()[0].confidence >= 0.9);
    }

    #[test]
    fn different_subjects_are_separate_memories() {
        let mut memory = WitnessMemory::default();
        memory.observe(WitnessKind::Violence, Some(1), 0, 1.0);
        memory.observe(WitnessKind::Violence, Some(2), 0, 1.0);
        assert_eq!(memory.len(), 2);
    }

    #[test]
    fn confidence_decays_with_absolute_time() {
        let mut memory = WitnessMemory::default();
        memory.observe(WitnessKind::Trespassing, None, 0, 1.0);
        memory.advance_to(10);
        let after_ten = memory.memories()[0].confidence;
        assert!(after_ten < 1.0);

        // Advancing to the same minute twice must not double-decay.
        memory.advance_to(10);
        assert_eq!(memory.memories()[0].confidence, after_ten);
    }

    #[test]
    fn weak_memories_are_forgotten() {
        let mut memory = WitnessMemory::default();
        memory.observe(WitnessKind::Trespassing, None, 0, 0.1);
        memory.advance_to(1000);
        assert!(memory.is_empty());
    }

    #[test]
    fn violence_is_remembered_longer_than_trespassing() {
        let mut memory = WitnessMemory::default();
        memory.observe(WitnessKind::Violence, Some(1), 0, 1.0);
        memory.observe(WitnessKind::Trespassing, None, 0, 1.0);
        memory.advance_to(50);
        let violence = memory
            .memories()
            .iter()
            .find(|m| m.kind == WitnessKind::Violence)
            .map(|m| m.confidence);
        assert!(violence.is_some());
        assert!(memory
            .memories()
            .iter()
            .all(|m| m.kind != WitnessKind::Trespassing));
    }

    #[test]
    fn capacity_is_bounded() {
        let mut memory = WitnessMemory {
            capacity: 2,
            ..WitnessMemory::default()
        };
        memory.observe(WitnessKind::Trespassing, None, 0, 0.2);
        memory.observe(WitnessKind::Blood, Some(1), 0, 0.5);
        memory.observe(WitnessKind::Violence, Some(2), 0, 1.0);
        assert_eq!(memory.len(), 2);
        assert_eq!(memory.strongest().unwrap().kind, WitnessKind::Violence);
    }

    #[test]
    fn weight_combines_confidence_and_severity() {
        let mut memory = WitnessMemory::default();
        memory.observe(WitnessKind::Violence, Some(1), 0, 1.0);
        memory.observe(WitnessKind::Trespassing, None, 0, 1.0);
        assert!((memory.total_weight() - 1.2).abs() < 1e-5);
        assert_eq!(memory.strongest().unwrap().kind, WitnessKind::Violence);
    }

    #[test]
    fn decay_never_goes_negative() {
        let mut item = Memory::new(WitnessKind::Violence, None, 0, 0.5);
        item.decay_to(100_000);
        assert_eq!(item.confidence, 0.0);
        assert!(item.is_forgotten());
    }

    #[test]
    fn time_going_backwards_does_not_increase_confidence() {
        let mut item = Memory::new(WitnessKind::Blood, None, 100, 0.5);
        item.decay_to(50);
        assert_eq!(item.confidence, 0.5);
    }
}

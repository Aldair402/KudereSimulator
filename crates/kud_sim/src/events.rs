//! Double-buffered simulation event queue.
//!
//! Systems write into the current buffer; consumers read the previous tick's
//! buffer. This removes order-dependent behavior where a system would react to
//! an event produced later in the same tick, which in the original depended on
//! MonoBehaviour execution order.

use crate::{GameEvent, StudentId};

/// Two-buffer event queue with deterministic ordering.
#[derive(Debug, Default)]
pub struct EventQueue {
    current: Vec<GameEvent>,
    previous: Vec<GameEvent>,
}

impl EventQueue {
    /// Creates an empty queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an event to the buffer being written this tick.
    #[inline]
    pub fn push(&mut self, event: GameEvent) {
        self.current.push(event);
    }

    /// Returns events produced during the current tick, in insertion order.
    pub fn current(&self) -> &[GameEvent] {
        &self.current
    }

    /// Returns events produced during the previous tick, in sorted order.
    pub fn previous(&self) -> &[GameEvent] {
        &self.previous
    }

    /// Number of events written so far this tick.
    pub fn current_len(&self) -> usize {
        self.current.len()
    }

    /// Sorts the current buffer and swaps it in as the readable buffer.
    ///
    /// Call once at the end of a tick. The retired buffer is cleared and reused,
    /// so a warm queue performs no allocation in steady state.
    pub fn flip(&mut self) {
        self.current.sort_by_key(event_order);
        self.previous.clear();
        std::mem::swap(&mut self.current, &mut self.previous);
    }

    /// Drains the readable buffer into a vector, leaving it empty.
    pub fn drain_previous(&mut self, out: &mut Vec<GameEvent>) {
        out.append(&mut self.previous);
    }

    /// Clears both buffers.
    pub fn clear(&mut self) {
        self.current.clear();
        self.previous.clear();
    }
}

/// Canonical event ordering key: student ID, then event kind, then payload.
///
/// Sorting by domain ID instead of entity handle keeps output stable across
/// spawn orders and archetype layouts.
fn event_order(event: &GameEvent) -> (StudentId, u8, u16) {
    match event {
        GameEvent::PhaseChanged { student_id, phase } => (*student_id, 0, u16::from(*phase)),
        GameEvent::StudentNoticedPlayer { student_id } => (*student_id, 1, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_readable_only_after_flip() {
        let mut queue = EventQueue::new();
        queue.push(GameEvent::StudentNoticedPlayer { student_id: 1 });
        assert!(queue.previous().is_empty());
        assert_eq!(queue.current_len(), 1);
        queue.flip();
        assert_eq!(queue.previous().len(), 1);
        assert_eq!(queue.current_len(), 0);
    }

    #[test]
    fn flip_sorts_by_student_id_then_kind() {
        let mut queue = EventQueue::new();
        queue.push(GameEvent::StudentNoticedPlayer { student_id: 2 });
        queue.push(GameEvent::PhaseChanged {
            student_id: 2,
            phase: 3,
        });
        queue.push(GameEvent::StudentNoticedPlayer { student_id: 1 });
        queue.flip();
        assert_eq!(
            queue.previous(),
            &[
                GameEvent::StudentNoticedPlayer { student_id: 1 },
                GameEvent::PhaseChanged {
                    student_id: 2,
                    phase: 3
                },
                GameEvent::StudentNoticedPlayer { student_id: 2 },
            ]
        );
    }

    #[test]
    fn previous_buffer_is_replaced_not_accumulated() {
        let mut queue = EventQueue::new();
        queue.push(GameEvent::StudentNoticedPlayer { student_id: 1 });
        queue.flip();
        queue.push(GameEvent::StudentNoticedPlayer { student_id: 2 });
        queue.flip();
        assert_eq!(
            queue.previous(),
            &[GameEvent::StudentNoticedPlayer { student_id: 2 }]
        );
    }

    #[test]
    fn empty_tick_clears_readable_buffer() {
        let mut queue = EventQueue::new();
        queue.push(GameEvent::StudentNoticedPlayer { student_id: 1 });
        queue.flip();
        queue.flip();
        assert!(queue.previous().is_empty());
    }

    #[test]
    fn drain_previous_empties_the_buffer() {
        let mut queue = EventQueue::new();
        queue.push(GameEvent::StudentNoticedPlayer { student_id: 4 });
        queue.flip();
        let mut out = Vec::new();
        queue.drain_previous(&mut out);
        assert_eq!(out.len(), 1);
        assert!(queue.previous().is_empty());
    }
}

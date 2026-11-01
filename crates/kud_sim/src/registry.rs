//! Stable domain IDs mapped to generational ECS entities.
//!
//! Original reference: `StudentManagerScript.Students`. The original indexed a
//! managed array by `StudentID`, which breaks when students are removed and makes
//! save data depend on array order. Here a `StudentId` is a permanent domain key
//! and the ECS handle behind it may change or be recycled.

use std::collections::BTreeMap;

use kud_ecs::Entity;

use crate::StudentId;

/// Bidirectional map between stable student IDs and ECS entities.
///
/// Iteration is ordered by `StudentId`, which keeps state hashing and event
/// emission independent of ECS archetype layout.
#[derive(Debug, Default)]
pub struct StudentRegistry {
    by_id: BTreeMap<StudentId, Entity>,
}

impl StudentRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds a student ID to an entity, returning the previous entity if any.
    pub fn insert(&mut self, student_id: StudentId, entity: Entity) -> Option<Entity> {
        self.by_id.insert(student_id, entity)
    }

    /// Returns the entity currently bound to a student ID.
    pub fn entity(&self, student_id: StudentId) -> Option<Entity> {
        self.by_id.get(&student_id).copied()
    }

    /// Removes a student ID binding and returns its entity.
    pub fn remove(&mut self, student_id: StudentId) -> Option<Entity> {
        self.by_id.remove(&student_id)
    }

    /// Returns true when the student ID is bound.
    pub fn contains(&self, student_id: StudentId) -> bool {
        self.by_id.contains_key(&student_id)
    }

    /// Returns the number of bound student IDs.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns true when no student IDs are bound.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Iterates bindings in ascending student ID order.
    pub fn iter(&self) -> impl Iterator<Item = (StudentId, Entity)> + '_ {
        self.by_id.iter().map(|(id, entity)| (*id, *entity))
    }

    /// Returns the lowest unused student ID at or above 1.
    ///
    /// ID 0 is reserved so an uninitialized field is never a valid student.
    pub fn next_free_id(&self) -> StudentId {
        let mut candidate = 1;
        for id in self.by_id.keys() {
            if *id > candidate {
                break;
            }
            candidate = *id + 1;
        }
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kud_ecs::World;

    #[test]
    fn inserts_and_resolves_ids() {
        let mut world = World::new();
        let entity = world.spawn((1_u32,));
        let mut registry = StudentRegistry::new();
        assert!(registry.insert(5, entity).is_none());
        assert_eq!(registry.entity(5), Some(entity));
        assert!(registry.contains(5));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn rebinding_returns_previous_entity() {
        let mut world = World::new();
        let first = world.spawn((1_u32,));
        let second = world.spawn((2_u32,));
        let mut registry = StudentRegistry::new();
        registry.insert(1, first);
        assert_eq!(registry.insert(1, second), Some(first));
        assert_eq!(registry.entity(1), Some(second));
    }

    #[test]
    fn iteration_is_sorted_by_id() {
        let mut world = World::new();
        let mut registry = StudentRegistry::new();
        for id in [9_u32, 2, 7, 1] {
            registry.insert(id, world.spawn((id,)));
        }
        let ids: Vec<_> = registry.iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![1, 2, 7, 9]);
    }

    #[test]
    fn next_free_id_skips_zero_and_gaps() {
        let mut world = World::new();
        let mut registry = StudentRegistry::new();
        assert_eq!(registry.next_free_id(), 1);
        registry.insert(1, world.spawn((1_u32,)));
        registry.insert(2, world.spawn((2_u32,)));
        assert_eq!(registry.next_free_id(), 3);
        registry.insert(5, world.spawn((5_u32,)));
        assert_eq!(registry.next_free_id(), 3);
        registry.insert(3, world.spawn((3_u32,)));
        registry.insert(4, world.spawn((4_u32,)));
        assert_eq!(registry.next_free_id(), 6);
    }

    #[test]
    fn removed_id_can_be_reused() {
        let mut world = World::new();
        let mut registry = StudentRegistry::new();
        let entity = world.spawn((1_u32,));
        registry.insert(3, entity);
        assert_eq!(registry.remove(3), Some(entity));
        assert!(!registry.contains(3));
        assert_eq!(registry.entity(3), None);
        assert!(registry.is_empty());
    }
}

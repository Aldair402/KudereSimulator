//! Entity-component-system layer used by the simulation.
//!
//! The engine owns its frame loop, asset model, renderer abstraction, and game
//! systems. `hecs` is used only as a small, auditable ECS storage library.

pub use hecs;
pub use hecs::{Entity, World};

/// Spawns a batch of entities and returns their identifiers.
pub fn spawn_batch<I, B>(world: &mut World, bundles: I) -> Vec<Entity>
where
    I: IntoIterator<Item = B>,
    B: hecs::Bundle + 'static,
{
    world.spawn_batch(bundles).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position(f32);

    #[test]
    fn spawns_and_queries_components() {
        let mut world = World::new();
        let entity = world.spawn((Position(1.0),));
        let mut query = world.query::<&Position>();
        let values: Vec<_> = query.iter().map(|(_, position)| position.0).collect();
        assert_eq!(values, vec![1.0]);
        assert!(world.contains(entity));
    }
}

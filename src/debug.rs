use bevy::prelude::*;
use crate::schedule::InGameSet;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, print_position.after(InGameSet::EntityUpdates));
    }
}

fn print_position(query: Query<(Entity, &Transform)>) {
    // log the Entity ID and transform of each entity with a 'transform' component.
    for (entity, transform) in query.iter() {
        let thing = entity.to_string();
        info!(
            "Entity {:?} is at transform {:?},",
            thing, transform.translation
        );
    }
}

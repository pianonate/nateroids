use bevy::prelude::*;

// provides a way to name entities that includes their entity id - for debugging
pub fn name_entity(commands: &mut Commands, entity: Entity, name: String) {
    commands
        .entity(entity)
        .insert(Name::new(format!("{} {}", name, entity)));
}

use bevy::prelude::*;

pub fn name_entity(commands: &mut Commands, entity: Entity, name: &str) {
    commands
        .entity(entity)
        .insert(Name::new(format!("{} {}", name, entity)));
}

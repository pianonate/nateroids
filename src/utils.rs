use bevy::prelude::*;

//noinspection Annotator
pub fn name_entity(commands: &mut Commands, entity: Entity, name: &str) {
    commands
        .entity(entity)
        .insert(Name::new(format!("{} {}", name, entity)));
}
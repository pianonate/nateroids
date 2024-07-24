use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;

pub const GROUP_SPACESHIP: Group = Group::GROUP_1;
pub const GROUP_ASTEROID: Group = Group::GROUP_2;
pub const GROUP_MISSILE: Group = Group::GROUP_3;

pub fn name_entity(commands: &mut Commands, entity: Entity, name: &str) {
    commands
        .entity(entity)
        .insert(Name::new(format!("{} {}", name, entity)));
}

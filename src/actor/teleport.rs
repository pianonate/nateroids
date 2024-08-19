use crate::{
    boundary::Boundary,
    schedule::InGameSet,
};
use bevy::prelude::*;

pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, teleport_at_boundary.in_set(InGameSet::EntityUpdates));
    }
}

#[derive(Component, Reflect, Debug, Default, Clone)]
pub struct Teleporter {
    pub just_teleported:          bool,
    pub last_teleported_position: Option<Vec3>,
    pub last_teleported_normal:   Option<Dir3>,
}

fn teleport_at_boundary(
    boundary: Res<Boundary>,
    mut wrappable_entities: Query<(&mut Transform, &mut Teleporter)>,
) {
    for (mut transform, mut wrappable) in wrappable_entities.iter_mut() {
        let original_position = transform.translation;

        let teleported_position = boundary.calculate_teleport_position(original_position);

        if teleported_position != original_position {
            transform.translation = teleported_position;
            wrappable.just_teleported = true;
            wrappable.last_teleported_position = Some(teleported_position);
            wrappable.last_teleported_normal = Some(boundary.get_normal_for_position(teleported_position));
        } else {
            wrappable.just_teleported = false;
            wrappable.last_teleported_position = None;
            wrappable.last_teleported_normal = None;
        }
    }
}

use bevy::prelude::*;

// no plugin yet because we don't need to do things like regenerate health
// so other entities will just use this component directly
#[derive(Component, Debug,)]
pub struct Health(pub f32,);

#[derive(Component, Debug,)]
pub struct CollisionDamage(pub f32,);

#[derive(Bundle,)]
pub struct HealthBundle {
    pub(crate) collision_damage: CollisionDamage,
    pub(crate) health:           Health,
}

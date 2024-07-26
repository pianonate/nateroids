use bevy::prelude::*;

// no plugin yet because we don't need to do things like regenerate health
// so other entities will just use this component directly
#[derive(Component, Debug)]
pub struct Health(pub f32);

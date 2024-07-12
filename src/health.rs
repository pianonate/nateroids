use bevy::prelude::*;

// no plugin yet because we don't need to do things like regenerate health
// so other entities will just use this component directly
#[derive(Component, Debug)]
pub struct Health {
    pub value: f32,
}

impl Health {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

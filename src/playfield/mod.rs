#[allow(clippy::module_inception)]
mod boundary;
mod planes;
mod wall_approach;

pub use crate::playfield::{
    boundary::Boundary,
    wall_approach::WallApproachVisual,
};

use crate::playfield::{
    boundary::BoundaryPlugin,
    planes::PlanesPlugin,
    wall_approach::WallApproachPlugin,
};
use bevy::prelude::*;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(WallApproachPlugin);
    }
}

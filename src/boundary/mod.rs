#[allow(clippy::module_inception)]
mod boundary;
mod planes;
mod wall_approach;

pub use crate::boundary::{
    boundary::Boundary,
    wall_approach::WallApproachVisual,
};

use crate::boundary::{
    boundary::BoundaryPlugin,
    planes::PlanesPlugin,
    wall_approach::WallApproachPlugin,
};
use bevy::prelude::*;

pub struct BoundaryModulePlugin;

impl Plugin for BoundaryModulePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(WallApproachPlugin);
    }
}

mod boundary_config;
mod planes;
mod visuals;
mod wall_approach;

pub use crate::boundary::{
    boundary_config::PlaneConfig,
    visuals::Boundary,
    wall_approach::WallApproachVisual,
};

use crate::boundary::{
    boundary_config::BoundaryConfigPlugin,
    planes::PlanesPlugin,
    visuals::VisualsPlugin,
    wall_approach::WallApproachPlugin,
};
use bevy::prelude::*;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryConfigPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(VisualsPlugin)
            .add_plugins(WallApproachPlugin);
    }
}

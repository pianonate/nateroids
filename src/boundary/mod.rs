mod boundary_config;
mod boundary_visuals;
mod planes;
mod wall_approach;

pub use crate::boundary::{
    boundary_config::BoundaryConfig,
    // todo: #bug - move PlaneConfig into its own .rs
    boundary_config::PlaneConfig,
    boundary_visuals::Boundary,
    wall_approach::WallApproachVisual,
};

use crate::boundary::{
    boundary_config::BoundaryConfigPlugin,
    boundary_visuals::BoundaryVisualsPlugin,
    planes::PlanesPlugin,
    wall_approach::WallApproachPlugin,
};
use bevy::prelude::*;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryConfigPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(BoundaryVisualsPlugin)
            .add_plugins(WallApproachPlugin);
    }
}

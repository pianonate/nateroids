#[allow(clippy::module_inception)]
mod boundary;
mod planes;
mod wall_portals;

pub use crate::playfield::{
    boundary::Boundary,
    wall_portals::WallApproachVisual,
};

use crate::playfield::{
    boundary::BoundaryPlugin,
    planes::PlanesPlugin,
    wall_portals::WallPortalPlugin,
};
use bevy::prelude::*;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(WallPortalPlugin);
    }
}

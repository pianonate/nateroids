mod boundary;
mod planes;
mod portals;
mod boundary_face;

pub use crate::playfield::{
    boundary::Boundary,
    portals::ActorPortals,
};

use crate::playfield::{
    boundary::BoundaryPlugin,
    planes::PlanesPlugin,
    portals::PortalPlugin,
};
use bevy::prelude::*;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(PortalPlugin);
    }
}

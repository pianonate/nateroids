use crate::state::GameState;
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum InGameSet {
    UserInput,
    EntityUpdates,
    CollisionDetection,
    DespawnEntities,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                InGameSet::DespawnEntities,
                // Flush commands (i.e. `apply_deferred` runs)
                InGameSet::UserInput,
                // a bug that if entity updates happen before collision detection
                // then the next despawn entites round gets rid of
                // new asteroids if the ship is also firing - weird
                // probably in the future i'll just use Rapier so i don't have
                // to concern myself with this
                //
                // it's got something to do with the missiles being fired
                // in any direction as that seems to cause a collision to be registered
                // with collision detection so that subsequently the newly created
                // asteroid is despawned
                //
                // for now putting collision detection before entity updates
                // so that newly spawned things don't get included in detection
                // which i suppose might make sense anyway...
                InGameSet::CollisionDetection,
                InGameSet::EntityUpdates,
            )
                .chain()
                // the following is pretty cool - because we added an InGameSet system set to
                // all the systems that are "in game" - in order to ensure proper ordering
                // the following comes along for the ride - i.e., they will only run _if_
                // if in_state evaluates to true - i.e., we are in_game
                // and we have a system that runs on state to watch for keyboard control
                // that takes us in or out of InGame - i.e., pausing
                // 1 line of code right here allows for pausing and starting the game
                // because we set it up that way
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            // apply_deferred - think of this as flushing all queued updates
            // in this case, after a despawn - before moving on to the next SystemSet
            // this way there isn't any chance that UserInput systems will use despawned entities
            // for performance reasons this is pretty cool
            apply_deferred
                .after(InGameSet::DespawnEntities)
                .before(InGameSet::UserInput),
        );
    }
}

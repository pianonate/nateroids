use crate::input::GlobalAction;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

// provides a way to name entities that includes their entity id - for debugging
pub fn name_entity(commands: &mut Commands, entity: Entity, name: String) {
    commands
        .entity(entity)
        .insert(Name::new(format!("{} {}", name, entity)));
}

#[derive(Default)]
pub struct ToggleState {
    pub state: bool,
}

// todo: #rustquestion, #bevyquesiton
//
// i couldn't have made this without gpt help - here's what it's telling me
//
// Each use of toggle_active() gets its own Local<ToggleState>.
//
// The Res<ActionState<GlobalAction>> is shared across the app, but each closure
// gets its own reference to it.
//
// Bevy's dependency injection system automatically provides these resources
// when the closure is executed, based on the types specified in the closure's
// signature.
//
// the impl Fn(..) piece is the key in that we're telling rust
// that "this function returns some type that implements the Fn(...) trait".
// so instead of a concrete type, we're specifying a trait that the
// returned type implements
// rust infers the actual concrete type based on the function body - in this
// case, a closure
// so:  toggle_active takes normal args and returns
//      * something that is a function (impl Fn)
//      * takes these two other params that bevy can dependency inject just like
//        systems
//      * returns a bool
//
// this is crazy to me
pub fn toggle_active(
    default: bool,
    action: GlobalAction,
) -> impl Fn(Res<ActionState<GlobalAction>>, Local<ToggleState>) -> bool {
    move |action_state: Res<ActionState<GlobalAction>>, mut state: Local<ToggleState>| {
        if action_state.just_pressed(&action) {
            state.state = !state.state;
        }

        if state.state {
            !default
        } else {
            default
        }
    }
}

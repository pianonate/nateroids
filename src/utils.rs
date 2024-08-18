use crate::input::GlobalAction;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use rand::Rng;
use std::ops::Range;

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

pub fn random_vec3(range_x: Range<f32>, range_y: Range<f32>, range_z: Range<f32>) -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = if range_x.start < range_x.end {
        rng.gen_range(range_x)
    } else {
        0.0
    };
    let y = if range_y.start < range_y.end {
        rng.gen_range(range_y)
    } else {
        0.0
    };
    let z = if range_z.start < range_z.end {
        rng.gen_range(range_z)
    } else {
        0.0
    };

    Vec3::new(x, y, z)
}

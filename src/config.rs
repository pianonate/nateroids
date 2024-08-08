use bevy::render::view::Layer;
use bevy::{
    color::{
        palettes::{css, tailwind},
        Color::Srgba,
    },
    prelude::*,
    render::view::RenderLayers,
};
use bevy_inspector_egui::InspectorOptions;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppearanceConfig>()
            .init_resource::<ColliderConfig>()
            .init_resource::<OrientationConfig>()
            .init_gizmo_group::<BoundaryGizmos>()
            .add_systems(Startup, init_gizmo_configs);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BoundaryGizmos {}

fn init_gizmo_configs(
    mut config_store: ResMut<GizmoConfigStore>,
    appearance_config: Res<AppearanceConfig>,
) {
    for (_, any_config, _) in config_store.iter_mut() {
        any_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
        any_config.line_width = 2.;
    }

    // so we can avoid an error of borrowing the mutable config_store twice
    // in the same context
    {
        let (config, _) = config_store.config_mut::<BoundaryGizmos>();
        config.line_width = appearance_config.boundary_line_width;
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct AppearanceConfig {
    pub ambient_light_brightness: f32,
    pub bloom_intensity: f32,
    pub bloom_low_frequency_boost: f32,
    pub bloom_high_pass_frequency: f32,
    pub boundary_color: Color,
    pub boundary_line_width: f32,
    pub boundary_cell_count: UVec3,
    pub boundary_cell_scalar: f32,
    pub clear_color: Color,
    pub clear_color_darkening_factor: f32,
    pub missile_forward_spawn_distance: f32,
    pub missile_circle_radius: f32,
    pub splash_timer: f32,
    pub zoom_sensitivity: f32,
}

// centralize appearance defaults
// plus this allows us to use the inspector to dynamically change them
// to try out different ratios while the game is running
impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            ambient_light_brightness: 3000.,
            bloom_intensity: 0.9,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.5,
            boundary_color: Color::from(tailwind::BLUE_300),
            boundary_line_width: 4.,
            boundary_cell_count: UVec3::new(2, 1, 1),
            boundary_cell_scalar: 110.,
            clear_color: Srgba(css::MIDNIGHT_BLUE),
            clear_color_darkening_factor: 0.019,
            missile_forward_spawn_distance: 5.6,
            missile_circle_radius: 7.,
            splash_timer: 2.,
            zoom_sensitivity: 8.,
        }
    }
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct ColliderConfig {
    pub missile: ColliderConstant,
    pub nateroid: ColliderConstant,
    pub spaceship: ColliderConstant,
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct ColliderConstant {
    pub name: &'static str,
    pub radius: f32,
    pub scalar: f32,
    pub spawnable: bool,
    pub velocity: f32,
}

// these scales were set by eye-balling the game
// if you get different assets these will likely need to change
// to match the assets size
impl Default for ColliderConfig {
    fn default() -> Self {
        Self {
            missile: ColliderConstant {
                name: "missile",
                radius: 0.5,
                scalar: 1.5,
                spawnable: true,
                velocity: 75.,
            },
            nateroid: ColliderConstant {
                name: "nateroid",
                radius: 2.3,
                scalar: 2.,
                spawnable: true,
                velocity: 30.,
            },
            spaceship: ColliderConstant {
                name: "spaceship",
                radius: 6.25,
                scalar: 0.8,
                spawnable: true,
                velocity: 40.,
            },
        }
    }
}
// centralize orientation defaults for a quick change-up
// Y-axis (vertical): Axis Mundi
// This represents the central axis of the world, connecting the heavens, earth, and underworld.
//
// X-axis (horizontal):
// Axis Orbis: Latin for "axis of the circle" or "axis of the world"
// This could represent the east-west movement of the sun or the horizon line.
//
// Z-axis (depth):
// Axis Profundus: Latin for "deep axis" or "profound axis"
// This could represent the concept of depth or the path between the observer and the horizon.
//
// nexus is the center of the game - It suggests a central point where all game elements connect
// or interact, which fits well with the concept of a game's core or hub.
//
// locus is the home position of the camera - It implies a specific, fixed point of reference,
// which aligns well with the idea of a camera's home or default position.
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct OrientationConfig {
    pub axis_mundi: Vec3,
    pub axis_orbis: Vec3,
    pub axis_profundus: Vec3,
    pub locus: Transform,
    pub nexus: Vec3,
}

impl Default for OrientationConfig {
    fn default() -> Self {
        Self {
            axis_mundi: Vec3::Y,
            axis_orbis: Vec3::X,
            axis_profundus: Vec3::Z,
            locus: Transform::default(),
            nexus: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct StarConfig {
    pub star_count: usize,
    pub star_radius: f32,
    pub star_field_inner_diameter: f32,
    pub star_field_outer_diameter: f32,
    pub star_spawn_batch_size: usize,
    pub twinkle_chance: f32,
    pub twinkle_duration_min: f32,
    pub twinkle_duration_max: f32,
    pub twinkle_intensity_min: f32,
    pub twinkle_intensity_max: f32,
    pub twinkle_per_update: usize,
}

//
impl Default for StarConfig {
    fn default() -> Self {
        Self {
            star_count: 5000,
            star_radius: 5.,
            star_field_inner_diameter: 1000.,
            star_field_outer_diameter: 20000.,
            star_spawn_batch_size: 50,
            twinkle_chance: 0.4, // percentage of stars to evaluate
            twinkle_duration_max: 2.,
            twinkle_duration_min: 0.2,
            twinkle_intensity_min: 10.0,
            twinkle_intensity_max: 40.,
            twinkle_per_update: 1, // stars to look at each update
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraOrder {
    Game,
    Stars,
}

impl CameraOrder {
    pub const fn order(self) -> isize {
        match self {
            CameraOrder::Game => 1,
            CameraOrder::Stars => 0,
        }
    }
}

// todo: how can i get PBRs to actually render on RenderLayer 1 so i could choose to have some
//       affected by bloom and some not...
// weird - if i put game on render layer 1 and stars on render layer 0,
// to line up with the camera order, the PBRs on render layer 1 are still
// showing on render layer 0 even though i don't think i asked for that

// used for both camera order and render layer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderLayer {
    Both,
    Game,
    Stars,
}

// returning the array rather than just one in case we have more complex
// situations in the future that require overlapping layers
impl RenderLayer {
    pub const fn layers(self) -> &'static [Layer] {
        match self {
            RenderLayer::Both => &[0, 1],
            RenderLayer::Game => &[0],
            RenderLayer::Stars => &[1],
        }
    }
}

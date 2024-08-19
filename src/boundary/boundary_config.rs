use bevy::{
    color::palettes::tailwind,
    prelude::*,
    render::render_resource::Face,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};

use crate::input::{
    toggle_active,
    GlobalAction,
};

pub struct BoundaryConfigPlugin;

// you have to register_type if you want the sliders to be created
impl Plugin for BoundaryConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoundaryConfig>()
            .register_type::<PlaneConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<BoundaryConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::BoundaryInspector)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<PlaneConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::PlanesInspector)),
            )
            .init_resource::<BoundaryConfig>()
            .init_resource::<PlaneConfig>();
    }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct BoundaryConfig {
    pub boundary_color:             Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub boundary_distance_approach: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub boundary_distance_shrink:   f32,
    #[inspector(min = 0.1, max = 10.0, display = NumberDisplay::Slider)]
    pub boundary_line_width:        f32,
    pub boundary_cell_count:        UVec3,
    #[inspector(min = 50., max = 200., display = NumberDisplay::Slider)]
    pub boundary_scalar:            f32,
    #[inspector(min = 1., max = 10., display = NumberDisplay::Slider)]
    pub smallest_teleport_circle:   f32,
}

impl Default for BoundaryConfig {
    fn default() -> Self {
        Self {
            boundary_color:             Color::from(tailwind::BLUE_300),
            boundary_distance_approach: 0.5,
            boundary_distance_shrink:   0.25,
            boundary_line_width:        4.,
            boundary_cell_count:        UVec3::new(2, 1, 1),
            boundary_scalar:            110.,
            smallest_teleport_circle:   5.,
        }
    }
}

// you can't use an #[inspector()] w/attenuation_distance
// because you have to use a logarithmic range to reach f32::INFINITY which is
// its default problem for another day...
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct PlaneConfig {
    pub front:                 bool,
    pub back:                  bool,
    pub top:                   bool,
    pub bottom:                bool,
    pub left:                  bool,
    pub right:                 bool,
    pub alpha_mode:            Option<AlphaMode>,
    pub base_color:            Color,
    #[reflect(ignore)]
    pub cull_mode:             Option<Face>,
    pub double_sided:          bool,
    pub emissive:              LinearRgba,
    pub attenuation_distance:  f32,
    #[inspector(min = 1.0, max = 3.0, display = NumberDisplay::Slider)]
    pub ior:                   f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub diffuse_transmission:  f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub metallic:              f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub perceptual_roughness:  f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub reflectance:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub specular_transmission: f32,
    #[inspector(min = 0.001, max = 10.0, display = NumberDisplay::Slider)]
    pub(crate) thickness:      f32,
}

impl Default for PlaneConfig {
    fn default() -> Self {
        Self {
            front:                 false,
            back:                  false,
            left:                  false,
            right:                 false,
            top:                   false,
            bottom:                false,
            alpha_mode:            None,
            attenuation_distance:  f32::INFINITY,
            base_color:            Color::from(LinearRgba::new(1., 1., 1., 1.)),
            cull_mode:             Some(Face::Back),
            diffuse_transmission:  0.,
            double_sided:          false,
            emissive:              LinearRgba::BLACK,
            ior:                   1.5,
            metallic:              0.,
            perceptual_roughness:  0.5,
            reflectance:           0.5,
            specular_transmission: 0.,
            thickness:             0.001,
        }
    }
}

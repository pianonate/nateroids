use crate::{
    camera::PrimaryCamera,
    config::AppearanceConfig,
    debug::{
        inspector_mode_enabled,
        InspectorMode,
    },
    input::GlobalAction,
    state::GameState,
};
use bevy::{
    diagnostic::{
        DiagnosticsStore,
        FrameTimeDiagnosticsPlugin,
    },
    prelude::*,
    render::camera::Viewport,
    window::PrimaryWindow,
};
use bevy_inspector_egui::{
    bevy_egui,
    bevy_egui::{
        EguiContext,
        EguiPlugin,
        EguiSet,
    },
    bevy_inspector,
    bevy_inspector::ui_for_state,
    egui,
    egui::{
        emath,
        Align,
        Layout,
    },
    DefaultInspectorConfigPlugin,
};
use egui_dock::{
    DockArea,
    DockState,
    NodeIndex,
};
use leafwing_input_manager::action_state::ActionState;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let default_config = AppearanceConfig::default();

        app.add_plugins(EguiPlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .insert_resource(UiState::new())
            .insert_resource(AmbientLightBrightness(
                default_config.ambient_light_brightness,
            ))
            .add_systems(
                PostUpdate,
                show_ui_system
                    .run_if(inspector_mode_enabled)
                    .before(EguiSet::ProcessOutput)
                    .before(TransformSystem::TransformPropagate),
            )
            .add_systems(
                PostUpdate,
                (
                    set_inspector_viewport.run_if(inspector_mode_enabled),
                    reset_camera_viewport.run_if(|button: Res<ActionState<GlobalAction>>| {
                        button.just_pressed(&GlobalAction::Inspector)
                    }),
                )
                    .chain()
                    .after(show_ui_system),
            );
    }
}

#[derive(Resource)]
pub struct AmbientLightBrightness(pub f32);

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

#[derive(Debug)]
enum EguiWindow {
    //RightSide,
    GameView,
    Inspector,
}

#[derive(Resource)]
struct UiState {
    state:         DockState<EguiWindow>,
    viewport_rect: egui::Rect,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        // let [game, _inspector] =
        //     tree.split_right(NodeIndex::root(), 0.8, vec![EguiWindow::RightSide]);
        // let [_game, _controls] = tree.split_left(game, 0.25,
        // vec![EguiWindow::Controls]);
        let [_game, _controls] =
            tree.split_left(NodeIndex::root(), 0.25, vec![EguiWindow::Inspector]);

        Self {
            state,
            viewport_rect: egui::Rect::NOTHING,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
        };
        DockArea::new(&mut self.state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

struct TabViewer<'a> {
    world:         &'a mut World,
    viewport_rect: &'a mut egui::Rect,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText { format!("{window:?}").into() }

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        match window {
            EguiWindow::Inspector => {
                let mut fps_value = 0.;
                let mut fps_average = 0.;

                if let Some(diagnostics) = self.world.get_resource::<DiagnosticsStore>() {
                    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                        if let Some(smoothed) = fps.smoothed() {
                            fps_value = smoothed;
                        }
                        if let Some(avg) = fps.average() {
                            fps_average = avg;
                        }
                    }
                };

                ui.label("Press F4 to toggle Inspector UI");
                ui.add_space(8.0);
                // Assuming you're in a `ui.horizontal` block
                // Assuming you're in a `ui.horizontal` block
                ui.horizontal(|ui| {
                    // Get the total available width
                    let total_width = ui.available_width();

                    // Split the width into three parts
                    let third_width = total_width / 3.0;

                    // Allocate the first third and right-align the label
                    ui.allocate_ui(emath::Vec2::new(third_width, ui.available_height()), |ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(format!("fps: {:>5.0}", fps_value));
                        });
                    });

                    // Allocate the second third and right-align the label
                    ui.allocate_ui(emath::Vec2::new(third_width, ui.available_height()), |ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(format!("avg: {:>5.0}", fps_average));
                        });
                    });

                    // Allocate the last third for any additional label later
                });
                ui.add_space(8.0);
                ui.label("GameState");
                ui_for_state::<GameState>(self.world, ui);
                ui.add_space(8.0);
                if let Some(mut ambient_light_value) =
                    self.world.get_resource_mut::<AmbientLightBrightness>()
                {
                    let label = "ambient light value:";
                    let min = 200.;
                    let max = 3000.;
                    let step_by = 200.;

                    ui.label(label);
                    ui.add(
                        egui::Slider::new(&mut ambient_light_value.0, min..=max)
                            .step_by(step_by)
                            .custom_formatter(|n, _| format!("{:.2}", n))
                            .custom_parser(|s| s.parse::<f64>().ok()),
                    );
                }
                egui::CollapsingHeader::new("entities")
                    .default_open(false)
                    .show(ui, |ui| {
                        bevy_inspector::ui_for_world_entities(self.world, ui);
                    });
                egui::CollapsingHeader::new("assets")
                    .default_open(false)
                    .show(ui, |ui| {
                        bevy_inspector::ui_for_all_assets(self.world, ui);
                    });
                egui::CollapsingHeader::new("resources")
                    .default_open(true)
                    .show(ui, |ui| {
                        bevy_inspector::ui_for_resources(self.world, ui);
                    });
            },
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();
            },
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool { false }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

// make camera render to full window
fn reset_camera_viewport(
    mut cameras: Query<&mut Camera, With<PrimaryCamera>>,
    inspector_mode: Res<InspectorMode>,
) {
    println!("resetting primary_camera_viewport running");

    if !inspector_mode.enabled {
        if let Ok(mut primary_camera) = cameras.get_single_mut() {
            primary_camera.viewport = None;
            println!("resetting primary_camera_viewport back to normal")
        }
    }
}

// make camera only render to view not obstructed by UI
// this accommodates window and tab resizes because it runs every frame
fn set_inspector_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut camera: Query<&mut Camera, With<PrimaryCamera>>,
) {
    if let Ok(mut primary_camera) = camera.get_single_mut() {
        if let Ok(window) = primary_window.get_single() {
            let viewport_rect = ui_state.viewport_rect;
            let scale_factor = window.scale_factor() * egui_settings.scale_factor;
            let viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor;
            let viewport_size = viewport_rect.size() * scale_factor;

            primary_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
                physical_size:     UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
                depth:             0.0..1.0,
            });

            //  println!("primary_camera.viewport: {:?}",
            // primary_camera.viewport);
        }
    }
}

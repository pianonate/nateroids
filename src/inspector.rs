use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};

use bevy_inspector_egui::{
    bevy_egui,
    bevy_egui::{EguiContext, EguiPlugin, EguiSet},
    bevy_inspector, egui, DefaultInspectorConfigPlugin,
};

use crate::{
    camera::PrimaryCamera,
    debug::{inspector_mode_enabled, DebugMode},
    state::GameState,
    window::{update_world_viewport_dimensions, ViewportData},
};

use bevy_inspector_egui::bevy_inspector::{ui_for_resource, ui_for_state};
use egui_dock::{DockArea, DockState, NodeIndex};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .insert_resource(UiState::new())
            .add_systems(
                PostUpdate,
                show_ui_system
                    .run_if(inspector_mode_enabled)
                    .before(EguiSet::ProcessOutput)
                    .before(TransformSystem::TransformPropagate),
            )
            .add_systems(
                PostUpdate,
                set_inspector_viewport
                    .run_if(inspector_mode_enabled)
                    .after(show_ui_system),
            )
            .add_systems(
                PostUpdate,
                reset_camera_viewport.run_if(not(inspector_mode_enabled)),
            );
    }
}

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
    Controls,
    GameView,
    Inspector,
}

#[derive(Resource)]
struct UiState {
    state: DockState<EguiWindow>,
    viewport_rect: egui::Rect,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.8, vec![EguiWindow::Inspector]);
        let [_game, _controls] = tree.split_left(game, 0.25, vec![EguiWindow::Controls]);

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
    world: &'a mut World,
    viewport_rect: &'a mut egui::Rect,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        format!("{window:?}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        match window {
            EguiWindow::Controls => {
                ui.label("Press F4 to toggle Inspector UI");
                ui.add_space(8.0);
                ui.label("GameState");
                ui_for_state::<GameState>(self.world, ui);
                ui.label("DebugMode");
                ui_for_resource::<DebugMode>(self.world, ui);
                ui.add_space(8.0);
                ui.label("DebugMode");
            }
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();
            }
            EguiWindow::Inspector => {
                egui::CollapsingHeader::new("entities")
                    .default_open(false)
                    .show(ui, |ui| {
                        bevy_inspector::ui_for_world_entities(self.world, ui);
                    });
                egui::CollapsingHeader::new("resources")
                    .default_open(false)
                    .show(ui, |ui| {
                        bevy_inspector::ui_for_resources(self.world, ui);
                    });
                egui::CollapsingHeader::new("assets")
                    .default_open(false)
                    .show(ui, |ui| {
                        bevy_inspector::ui_for_all_assets(self.world, ui);
                    });
            }
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

// make camera render to full window
// todo: #bevyquestion - set_normal_viewport run every update
//                       seems as if it only need to run once, when the egui is toggled off
fn reset_camera_viewport(mut cameras: Query<&mut Camera, With<PrimaryCamera>>) {
    let mut primary_camera = cameras.single_mut();
    primary_camera.viewport = None;
}

// make camera only render to view not obstructed by UI
// this accommodates window and tab resizes because it runs every frame
fn set_inspector_viewport(
    mut commands: Commands,
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut camera: Query<(&mut Camera, &Projection, &GlobalTransform), With<PrimaryCamera>>,
) {
    if let Ok((
        mut primary_camera,
        Projection::Perspective(perspective_projection),
        global_transform,
    )) = camera.get_single_mut()
    {
        if let Ok(window) = primary_window.get_single() {
            let viewport_rect = ui_state.viewport_rect;
            let scale_factor = window.scale_factor() * egui_settings.scale_factor;
            let viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor;
            let viewport_size = viewport_rect.size() * scale_factor;

            primary_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
                physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
                depth: 0.0..1.0,
            });

            let viewport_data = ViewportData {
                fov: perspective_projection.fov,
                camera_distance: global_transform.translation().z,
                height: viewport_size.y,
                width: viewport_size.x,
            };

            update_world_viewport_dimensions(&mut commands, viewport_data);
        }
    }
}

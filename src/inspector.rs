use crate::camera::PrimaryCamera;

use bevy::{
    asset::{ReflectAsset, UntypedAssetId},
    input::common_conditions::input_toggle_active,
    prelude::*,
    reflect::TypeRegistry,
    render::camera::Viewport,
    window::PrimaryWindow,
};

use bevy_inspector_egui::{
    bevy_egui,
    bevy_egui::{EguiContext, EguiPlugin, EguiSet},
    bevy_inspector,
    bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities},
    bevy_inspector::{ui_for_entities_shared_components, ui_for_entity_with_children},
    egui, DefaultInspectorConfigPlugin,
};
use egui_dock::{DockArea, DockState, NodeIndex};
use std::any::TypeId;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(DefaultInspectorConfigPlugin)
            .insert_resource(UiState::new())
            .add_systems(
                PostUpdate,
                show_ui_system
                    .run_if(input_toggle_active(false, KeyCode::F3))
                    .before(EguiSet::ProcessOutput)
                    .before(TransformSystem::TransformPropagate),
            )
            .add_systems(
                PostUpdate,
                set_inspector_viewport
                    .run_if(input_toggle_active(false, KeyCode::F3))
                    .after(show_ui_system),
            )
            .add_systems(
                PostUpdate,
                set_normal_viewport.run_if(input_toggle_active(true, KeyCode::F3)),
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

// fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
//     let mut egui_context = world
//         .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
//         .single(world)
//         .clone();
//     egui::SidePanel::left("hierarchy")
//         .default_width(200.0)
//         .show(egui_context.get_mut(), |ui| {
//             egui::ScrollArea::both().show(ui, |ui| {
//                 ui.label("Press F3 to toggle Inspector UI");
//                 ui.add_space(8.0);
//                 ui.label("GameState");
//                 ui_for_state::<GameState>(world, ui);
//                 ui.label("DebugMode");
//                 ui_for_resource::<DebugMode>(world, ui);
//                 ui.add_space(8.0);
//
//                 ui.heading("Hierarchy");
//
//                 bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
//                     world,
//                     ui,
//                     &mut selected_entities,
//                 );
//
//                 ui.allocate_space(ui.available_size());
//             });
//         });
//
//     egui::SidePanel::right("inspector")
//         .default_width(250.0)
//         .show(egui_context.get_mut(), |ui| {
//             egui::ScrollArea::both().show(ui, |ui| {
//                 ui.heading("Inspector");
//
//                 match selected_entities.as_slice() {
//                     &[entity] => {
//                         bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
//                     }
//                     entities => {
//                         bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
//                             world, entities, ui,
//                         );
//                     }
//                 }
//
//                 ui.allocate_space(ui.available_size());
//             });
//         });
// }

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Debug)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}

#[derive(Resource)]
struct UiState {
    state: DockState<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.8, vec![EguiWindow::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.25, vec![EguiWindow::Hierarchy]);
        let [_game, _bottom] =
            tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            state,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
        };
        DockArea::new(&mut self.state)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        format!("{window:?}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            },
        }
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn set_normal_viewport(
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<PrimaryCamera>>,
) {
    let mut primary_camera = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    primary_camera.viewport = Some(Viewport {
        physical_position: UVec2::ZERO,
        physical_size: UVec2::new(window.width() as u32, window.height() as u32),
        depth: 0.0..1.0,
    });
}

// make camera only render to view not obstructed by UI
fn set_inspector_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<PrimaryCamera>>,
) {
    let mut primary_camera = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let viewport_rect = ui_state.viewport_rect;
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;
    let viewport_pos = viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = viewport_rect.size() * scale_factor;

    primary_camera.viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}

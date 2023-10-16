use std::any::TypeId;

use bevy::{
    prelude::*,
    asset::{
        HandleId, ReflectAsset,
    },
    reflect::TypeRegistry,
    render::camera::Viewport,
    window::PrimaryWindow,
};

use bevy_egui::{
    EguiContext,
    EguiContexts,
    EguiPlugin,
    EguiSet,
};
use bevy_inspector_egui::{
    bevy_inspector::{
        hierarchy::{
            hierarchy_ui,
            SelectedEntities
        },
        self,
        ui_for_entities_shared_components,
        ui_for_entity_with_children,
    },
    DefaultInspectorConfigPlugin,
};
use bevy_pancam::{
    PanCam,
    PanCamPlugin,
    PanCamSystemSet,
};
use egui_dock::{
    DockArea,
    NodeIndex,
    Style,
    Tree
};


// TODO: toggle UI with F1 key
// TODO: move UI system to core as a plugin, expose AutomataReflect to filter UI (expect proper type registration still), draw fps over UI, or move it into clip-rect space?
#[derive(Default)]
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultInspectorConfigPlugin,
            EguiPlugin,
            PanCamPlugin,
        ));

        app.init_resource::<EguiWantsFocus>();
        app.insert_resource(UiState::new());

        app.add_systems(Update, esc_close);
        app.add_systems(
            PostUpdate,
            show_ui_system
                .before(EguiSet::ProcessOutput)
                .before(bevy::transform::TransformSystem::TransformPropagate),
        );
        app.add_systems(PostUpdate, set_camera_viewport.after(show_ui_system));
        app.add_systems(Startup, setup_camera);

        app.configure_set(
            Update,
            PanCamSystemSet.run_if(resource_equals(EguiWantsFocus(false))),
        );

        // TODO: try removing these
        app.register_type::<Option<Handle<Image>>>();
        app.register_type::<AlphaMode>();
    }
}

fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
        PanCam::default(),
    ));
}


fn esc_close(
    keys: Res<Input<KeyCode>>,
    mut ui_state: ResMut<UiState>,
) {
    if keys.just_pressed(KeyCode::F1) {
        ui_state.enabled = !ui_state.enabled;
    }
}


#[derive(Component)]
struct MainCamera;

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


#[derive(Resource, Deref, DerefMut, PartialEq, Eq, Default)]
struct EguiWantsFocus(bool);

fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    mut cameras: Query<&mut Camera, With<MainCamera>>,
    mut contexts: EguiContexts,
    mut wants_focus: ResMut<EguiWantsFocus>,
) {
    let mut cam = cameras.single_mut();

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    if !ui_state.enabled {
        cam.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(window.width() as u32, window.height() as u32),
            depth: 0.0..1.0,
        });
        return;
    }

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor as f32;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor as f32;

    cam.viewport = Some(Viewport {
        physical_position: UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32),
        physical_size: UVec2::new(viewport_size.x as u32, viewport_size.y as u32),
        depth: 0.0..1.0,
    });

    let pointer_pos = contexts.ctx_mut().pointer_latest_pos();
    if let Some(pointer_pos) = pointer_pos {
        let pointer_over_render_view = viewport_pos.x <= pointer_pos.x
            && pointer_pos.x <= viewport_pos.x + viewport_size.x
            && viewport_pos.y <= pointer_pos.y
            && pointer_pos.y <= viewport_pos.y + viewport_size.y;

        let new_wants_focus = (contexts.ctx_mut().wants_keyboard_input() || contexts.ctx_mut().wants_pointer_input()) && !pointer_over_render_view;
        wants_focus.set_if_neq(EguiWantsFocus(new_wants_focus));
    }
}


#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, HandleId),
}

#[derive(Resource)]
struct UiState {
    tree: Tree<EguiWindow>,
    viewport_rect: egui::Rect,
    selected_entities: SelectedEntities,
    selection: InspectorSelection,
    enabled: bool,
}

impl UiState {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec![EguiWindow::GameView]);
        let [game, _inspector] = tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
        let [_game, _bottom] = tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            tree,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
            enabled: false,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        if !self.enabled {
            return;
        }

        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
        };
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

#[derive(Debug)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
}

// TODO: redo the UI to be more CA focused (only select CA types tagged with AutomataReflect?)
impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();

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
                        &type_registry.internal.read(),
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry.internal.read(),
                    );
                }
            },
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry.read()
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| (registration.short_name().to_owned(), registration.type_id()))
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, &resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name);
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let type_registry = type_registry.read();
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.short_name().to_owned(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let mut handles: Vec<_> = reflect_asset.ids(world).collect();
        handles.sort();

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
                        InspectorSelection::Asset(asset_type_id, asset_name.clone(), handle);
                }
            }
        });
    }
}

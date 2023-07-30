use std::any::TypeId;

use bevy::{
    prelude::*,
    asset::{
        HandleId, ReflectAsset,
    },
    reflect::{TypeUuid, TypeRegistry},
    render::{
        render_resource::{
            AsBindGroup,
            Extent3d,
            ShaderRef,
        },
        camera::Viewport,
    },
    sprite::Material2d,
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

use num_format::{Locale, ToFormattedString};

use rusty_automata::{
    RustyAutomataApp,
    automata::{
        AutomataField,
        AutomataPlugin,
    },
    neat::{
        NeatField,
        NeatPlugin,
    },
    utils::setup_hooks,
};


fn example_app() {
    App::new()
        .add_plugins((
            RustyAutomataApp::default(),
            AutomataPlugin::default(),
            NeatPlugin::default(),
        ))
        .add_plugins((
            DefaultInspectorConfigPlugin,
            EguiPlugin,
        ))
        .add_plugins((
            PanCamPlugin::default(),
        ))
        .insert_resource(UiState::new())
        .add_systems(
            PostUpdate,
            show_ui_system
                .before(EguiSet::ProcessOutput)
                .before(bevy::transform::TransformSystem::TransformPropagate),
        )
        .init_resource::<EguiWantsFocus>()
        .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system))
        .configure_set(
            Update,
            PanCamSystemSet.run_if(resource_equals(EguiWantsFocus(false))),
        )
        .add_systems(Startup, setup)
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>()
        .run();
}


fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    // TODO: pull from config/UI

    let window = windows.single();
    let field_size = Extent3d {
        width: window.resolution.width() as u32,
        height: window.resolution.height() as u32,
        depth_or_array_layers: 1,
    };

    // TODO: change to creation args struct
    let edge_count: u32 = 25;

    let automata_field = AutomataField::new(
        field_size,
        edge_count,
        &mut images
    );
    let neat_field = NeatField::new(field_size, &mut images);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.resolution.width() as f32, window.resolution.height() as f32)),
            ..default()
        },
        texture: automata_field.nodes.clone(),
        ..default()
    });

    commands.insert_resource(automata_field);
    commands.insert_resource(neat_field);

    // TODO(test): add visual remap layer via fragment shader
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
        PanCam::default(),
    ));

    println!("field_size: {:?}x{:?}", field_size.width, field_size.height);
    let parameters = (field_size.width * field_size.height * 8 + edge_count * 4) * field_size.depth_or_array_layers;
    println!("parameters: {}", parameters.to_formatted_string(&Locale::en));
}


// TODO: move UI system to core as a plugin, expose AutomataReflect to filter UI (expect proper type registration still), draw fps over UI, or move it into clip-rect space?
// TODO: toggle UI with F1 key
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

// make camera only render to view not obstructed by UI
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
}

impl UiState {
    pub fn new() -> Self {
        let mut tree = Tree::new(vec![EguiWindow::GameView]);
        let [game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![EguiWindow::Inspector]);
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
        let [_game, _bottom] =
            tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            tree,
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


#[derive(AsBindGroup, Clone, Debug, Default, Reflect, TypeUuid)]
#[uuid = "ac2f08eb-5234-1262-5556-51571ea332d5"]
struct NeatVisualMaterial { }

impl Material2d for NeatVisualMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/neat.wgsl".into()
    }
}


pub fn main() {
    setup_hooks();
    example_app();
}

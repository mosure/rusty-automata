use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_resource::{
            AsBindGroup,
            Extent3d,
            ShaderRef,
        },
    },
    sprite::{
        Material2d,
        Material2dPlugin,
        MaterialMesh2dBundle
    },
};

use noisy_bevy::NoisyShaderPlugin;

use rusty_automata::{
    RustyAutomataApp,
    plot::PlotPlugin,
    uaf::UafPlugin,
    utils::setup_hooks,
};


fn example_app() {
    App::new()
        .add_plugin(RustyAutomataApp::default())
        .add_plugin(NoisyShaderPlugin)
        .add_plugin(PlotPlugin)
        .add_plugin(UafPlugin)
        .add_plugin(Material2dPlugin::<UafMaterial>::default())
        .add_startup_system(setup_screen)
        .run();
}


fn setup_screen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut uaf_materials: ResMut<Assets<UafMaterial>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        size.width as f32,
        size.height as f32,
    ))));

    let material_handle = uaf_materials.add(UafMaterial {});

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
    ));

    commands.spawn((
        Camera2dBundle {
            ..default()
        },
    ));

    // TODO: add UI controls for a, b, c, d of UAF
}


#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "ac2f08eb-67fa-23f1-a908-51571ea332d5"]
struct UafMaterial { }

impl Material2d for UafMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/uaf.wgsl".into()
    }
}


pub fn main() {
    setup_hooks();
    example_app();
}

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{
        AsBindGroup,
        Extent3d,
        ShaderRef,
    },
    sprite::{
        Material2d,
        Material2dPlugin,
        MaterialMesh2dBundle
    },
};

use rusty_automata::{
    RustyAutomataApp,
    utils::setup_hooks,
};


fn example_app() {
    App::new()
        .add_plugins((
            RustyAutomataApp::default(),
            Material2dPlugin::<SandboxMaterial>::default(),
        ))
        .register_asset_reflect::<SandboxMaterial>()
        .add_systems(Startup, setup_screen)
        .run();
}


fn setup_screen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut uaf_materials: ResMut<Assets<SandboxMaterial>>,
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

    let material_handle = uaf_materials.add(SandboxMaterial::default());

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
}


// TODO: figure out why material asset handle ID is displayed instead of the material name
#[derive(AsBindGroup, Clone, Debug, Default, Reflect, TypeUuid)]
#[uuid = "ac2f08eb-67fa-23f1-a908-51571ea332d5"]
struct SandboxMaterial { }

impl Material2d for SandboxMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sandbox.wgsl".into()
    }
}

pub fn main() {
    setup_hooks();
    example_app();
}

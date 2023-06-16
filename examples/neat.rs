use bevy::{
    core_pipeline::blit::{
        BlitPipeline,
        BlitPipelineKey,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::RenderTarget,
        render_resource::{
            AsBindGroup,
            Extent3d,
            ShaderRef,
            TextureDescriptor,
            TextureDimension,
            TextureFormat,
            TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::{
        Material2d,
        Material2dPlugin,
        MaterialMesh2dBundle,
    },
};

use rusty_automata::{
    RustyAutomataApp,
    utils::setup_hooks,
};


fn example_app() {
    App::new()
        .add_plugin(RustyAutomataApp::default())
        .add_plugin(Material2dPlugin::<NeatMaterial>::default())
        .add_startup_system(setup)
        .run();
}


#[derive(Component)]
struct StatePass;

#[derive(Component)]
struct RenderPass;



// TODO: majority of game of life example can be used: https://github.com/bevyengine/bevy/blob/main/examples/shader/compute_shader_game_of_life.rs#L64
// however, it may be more efficient to implement in fragment mode




fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut error_function_materials: ResMut<Assets<NeatMaterial>>,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
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

    let material_handle = error_function_materials.add(NeatMaterial {});

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size); // fill image.data with zeroes

    let image_handle = images.add(image);


    let first_pass_layer = RenderLayers::layer(1);


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


#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "ac2f08eb-67fb-23f1-a908-54871ea597d5"]
struct NeatMaterial { }

impl Material2d for NeatMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/neat.wgsl".into()
    }
}


pub fn main() {
    setup_hooks();
    example_app();
}

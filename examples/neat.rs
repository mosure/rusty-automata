use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{
        AsBindGroup,
        Extent3d,
        ShaderRef,
    },
    sprite::Material2d,
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
        .add_systems(Startup, setup)
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

    println!("field_size: {:?}x{:?}", field_size.width, field_size.height);
    let parameters = (field_size.width * field_size.height * 8 + edge_count * 4) * field_size.depth_or_array_layers;
    println!("parameters: {}", parameters.to_formatted_string(&Locale::en));
}


// TODO: add visual remap layer via fragment shader
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

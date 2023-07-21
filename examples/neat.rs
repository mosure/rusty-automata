use bevy::{
    prelude::*,
    render::render_resource::Extent3d,
};

use num_format::{Locale, ToFormattedString};

use rusty_automata::{
    RustyAutomataApp,
    automata::AutomataField,
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
            NeatPlugin,
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

    let edge_neighborhood: u32 = 5;

    let automata_field = AutomataField::new(
        field_size,
        edge_neighborhood,
        &mut images
    );

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.resolution.width() as f32, window.resolution.height() as f32)),
            ..default()
        },
        texture: automata_field.nodes.clone(),
        ..default()
    });

    commands.insert_resource(automata_field);
    commands.insert_resource(NeatField::new(field_size, &mut images));

    // TODO: add visual remap layer via fragment shader
    commands.spawn(Camera2dBundle::default());

    println!("field_size: {:?}x{:?}", field_size.width, field_size.height);
    let parameters = field_size.width * field_size.height * 7 + edge_neighborhood * edge_neighborhood * 4;
    println!("parameters: {}", parameters.to_formatted_string(&Locale::en));
}


pub fn main() {
    setup_hooks();
    example_app();
}

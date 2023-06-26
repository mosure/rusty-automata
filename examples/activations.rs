use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d,
        },
    },
};

use rusty_automata::{
    RustyAutomataApp,
    activations::generate_activation_texture,
    utils::setup_hooks,
};


fn example_app() {
    App::new()
        .add_plugin(RustyAutomataApp::default())
        .add_startup_system(setup)
        .run();
}


fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = windows.single();
    let size = Extent3d {
        width: window.resolution.width() as u32,
        height: window.resolution.height() as u32,
        depth_or_array_layers: 1,
    };

    let activations = generate_activation_texture();
    let image = images.add(activations);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(size.width as f32, size.height as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());
}


pub fn main() {
    setup_hooks();
    example_app();
}

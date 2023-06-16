use bevy::{
    prelude::*,
};

use rusty_automata::{
    RustyAutomataApp,
    utils::setup_hooks,
};


fn example_app() {
    App::new()
        .add_plugin(RustyAutomataApp::default())
        .add_startup_system(help_display_setup)
        .run();
}


pub fn main() {
    setup_hooks();
    example_app();
}



fn help_display_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(NodeBundle {
        style: Style {
            size: Size::width(Val::Percent(100.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        parent.spawn(
            (
                TextBundle::from_sections([
                    TextSection::new(
                        "cargo run --example {name}",
                        TextStyle {
                            font: asset_server.load("fonts/Caveat-Bold.ttf"),
                            font_size: 60.0,
                            color: Color::WHITE,
                        },
                    ),
                ]),
            )
        );
    });

    commands.spawn((
        Camera2dBundle {
            ..default()
        },
    ));
}

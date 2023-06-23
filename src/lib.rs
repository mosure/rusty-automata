use bevy::{
    prelude::*,
    app::AppExit,
    diagnostic::{
        Diagnostics,
        FrameTimeDiagnosticsPlugin,
    },
};
use bevy_framepace::{
    FramepaceSettings,
    Limiter,
};

pub mod utils;


pub struct RustyAutomataApp {
    esc_close: bool,
    fps_limit: f64,
    show_fps: bool,
    width: f32,
    height: f32,
    name: String,
}

impl Default for RustyAutomataApp {
    fn default() -> RustyAutomataApp {
        RustyAutomataApp {
            esc_close: true,
            fps_limit: 0.0,
            show_fps: true,
            width: 1920.0,
            height: 1080.0,
            name: "rusty automata".to_string(),
        }
    }
}

impl Plugin for RustyAutomataApp {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: false,
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    prevent_default_event_handling: false,
                    resolution: (self.width, self.height).into(),
                    title: self.name.clone(),
                    ..default()
                }),
                ..default()
            }));

        if self.esc_close {
            app.add_system(esc_close);
        }

        if self.fps_limit > 0.0 {
            app.add_plugin(bevy_framepace::FramepacePlugin);

            let fps_limit = self.fps_limit;
            app.add_startup_system(move |settings: ResMut<FramepaceSettings>| {
                fps_throttle_setup(settings, fps_limit);
            });
        }

        if self.show_fps {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default());
            app.add_startup_system(fps_display_setup);
            app.add_system(fps_update_system);
        }
    }
}


pub fn esc_close(
    keys: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}


fn fps_throttle_setup(
    mut settings: ResMut<FramepaceSettings>,
    fps: f64,
) {
    settings.limiter = Limiter::from_framerate(fps);
}


fn fps_display_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "fps: ",
                TextStyle {
                    font: asset_server.load("fonts/Caveat-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/Caveat-Medium.ttf"),
                font_size: 60.0,
                color: Color::GOLD,
            }),
        ]).with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        FpsText,
    ));
}

#[derive(Component)]
struct FpsText;

fn fps_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

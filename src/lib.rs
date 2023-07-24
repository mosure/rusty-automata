use bevy::{
    prelude::*,
    app::AppExit,
    diagnostic::{
        DiagnosticsStore,
        FrameTimeDiagnosticsPlugin,
    },
    render::{
        RenderPlugin,
        settings::{
            WgpuLimits,
            WgpuSettings,
        },
    },
};
// TODO: update to latest framepace
// use bevy_framepace::{
//     FramepaceSettings,
//     Limiter,
// };

use noise::NoisePlugin;

// TODO: move to crate project structure
pub mod automata;
pub mod neat;
pub mod noise;
pub mod plot;
pub mod uaf;
pub mod utils;


pub struct RustyAutomataApp {
    esc_close: bool,
    //fps_limit: f64,
    show_fps: bool,
    width: f32,
    height: f32,
    name: String,
}

impl Default for RustyAutomataApp {
    fn default() -> RustyAutomataApp {
        RustyAutomataApp {
            esc_close: true,
            //fps_limit: 0.0,
            show_fps: true,
            width: 1920.0,
            height: 1080.0,
            name: "rusty automata".to_string(),
        }
    }
}

impl Plugin for RustyAutomataApp {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb_u8(112, 48, 48)));
        app.add_plugins(
            DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(RenderPlugin {
                wgpu_settings: WgpuSettings {
                    limits: WgpuLimits {
                        //max_texture_dimension_2d: 16384, // TODO: use 2d texture array for tiling fields
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: false,
                    mode: bevy::window::WindowMode::Windowed,
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    prevent_default_event_handling: false,
                    resolution: (self.width, self.height).into(),
                    title: self.name.clone(),
                    ..default()
                }),
                ..default()
            })
        );
        app.add_plugins(
            NoisePlugin,
        );

        if self.esc_close {
            app.add_systems(Update, esc_close);
        }

        // if self.fps_limit > 0.0 {
        //     app.add_plugin(bevy_framepace::FramepacePlugin);

        //     let fps_limit = self.fps_limit;
        //     app.add_startup_system(move |settings: ResMut<FramepaceSettings>| {
        //         fps_throttle_setup(settings, fps_limit);
        //     });
        // }

        if self.show_fps {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
            app.add_systems(Startup, fps_display_setup);
            app.add_systems(Update, fps_update_system);
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


// fn fps_throttle_setup(
//     mut settings: ResMut<FramepaceSettings>,
//     fps: f64,
// ) {
//     settings.limiter = Limiter::from_framerate(fps);
// }


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
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        FpsText,
    ));
}

#[derive(Component)]
struct FpsText;

fn fps_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

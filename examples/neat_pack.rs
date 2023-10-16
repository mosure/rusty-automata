use bevy::prelude::*;

use rusty_automata::{
    RustyAutomataApp,
    neat::pack::{
        NeatEdge,
        NeatGraph,
        NeatNode,
        NeatPopulation,
        NeatUafActivation,
        population_to_textures,
    },
    utils::setup_hooks,
};

fn example_app() {
    App::new()
        .add_plugins((
            RustyAutomataApp::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}


fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    let graphs: Vec<NeatGraph> = vec![
        NeatGraph {
            nodes: vec![
                NeatNode {
                    activation: NeatUafActivation {
                        a: 0.5,
                        b: 0.0,
                        c: 0.5,
                        d: 1.0,
                        e: 0.0,
                    },
                    source_edges: vec![
                        NeatEdge {
                            weight: 1.0,
                            source: 1,
                        },
                    ],
                },
                NeatNode {
                    activation: NeatUafActivation {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                    },
                    source_edges: vec![
                        NeatEdge {
                            weight: 1.0,
                            source: 1,
                        },
                    ],
                },
                NeatNode {
                    activation: NeatUafActivation {
                        a: 0.0,
                        b: 1.0,
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                    },
                    source_edges: vec![
                        NeatEdge {
                            weight: 1.0,
                            source: 2,
                        },
                    ],
                },
                NeatNode {
                    activation: NeatUafActivation {
                        a: 1.0,
                        b: 1.0,
                        c: 0.0,
                        d: 1.0,
                        e: 0.0,
                    },
                    source_edges: vec![
                        NeatEdge {
                            weight: 1.0,
                            source: 3,
                        },
                    ],
                },
            ],
        },
        NeatGraph {
            nodes: vec![
                NeatNode {
                    activation: NeatUafActivation {
                        a: 0.0,
                        b: 0.0,
                        c: 1.0,
                        d: 1.0,
                        e: 0.0,
                    },
                    source_edges: vec![
                        NeatEdge {
                            weight: 1.0,
                            source: 0,
                        },
                    ],
                },
                NeatNode {
                    activation: NeatUafActivation {
                        a: 1.0,
                        b: 1.0,
                        c: 1.0,
                        d: 0.5,
                        e: 0.0,
                    },
                    source_edges: vec![
                        NeatEdge {
                            weight: 1.0,
                            source: 0,
                        },
                    ],
                },
            ],
        },
    ];

    let population = NeatPopulation {
        graphs,
    };
    let textures = population_to_textures(&population);

    let activations = images.add(textures.activations);
    let edges = images.add(textures.edges);
    let nodes = images.add(textures.nodes);

    let window = windows.single();
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.resolution.width() as f32, window.resolution.height() as f32)),
            ..default()
        },
        texture: activations,
        ..default()
    });
}

pub fn main() {
    setup_hooks();
    example_app();
}

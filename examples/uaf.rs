use bevy::{
    prelude::*,
    reflect::{
        TypeUuid
    },
    render::{
        render_asset::{
            RenderAssets,
        },
        render_resource::{
            AsBindGroup,
            AsBindGroupShaderType,
            Extent3d,
            ShaderRef,
            ShaderType,
        },
    },
    sprite::{
        Material2d,
        Material2dPlugin,
        MaterialMesh2dBundle
    },
};
use bevy_inspector_egui::{
    InspectorOptions,
    inspector_options::std_options::NumberDisplay,
    prelude::{
        ReflectInspectorOptions,
    },
    quick::{
        AssetInspectorPlugin,
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
        .register_asset_reflect::<UafMaterial>()
        .add_plugin(AssetInspectorPlugin::<UafMaterial>::default())
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

    let material_handle = uaf_materials.add(UafMaterial::default());

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
#[derive(AsBindGroup, Clone, Debug, FromReflect, InspectorOptions, Reflect, TypeUuid)]
#[reflect(Debug, Default, InspectorOptions)]
#[uuid = "ac2f08eb-67fa-23f1-a908-51571ea332d5"]
#[uniform(0, UafMaterialUniform)]
struct UafMaterial {
    #[inspector(min = -1.0, max = 1.0, display = NumberDisplay::Slider)]
    a: f32,
    #[inspector(min = -1.0, max = 1.0, display = NumberDisplay::Slider)]
    b: f32,
    #[inspector(min = -1.0, max = 1.0, display = NumberDisplay::Slider)]
    c: f32,
    #[inspector(min = -1.0, max = 1.0, display = NumberDisplay::Slider)]
    d: f32,
    #[inspector(min = -1.0, max = 1.0, display = NumberDisplay::Slider)]
    e: f32,
    animate: bool,
}

impl Default for UafMaterial {
    fn default() -> Self {
        // default to sigmoid - https://arxiv.org/pdf/2011.03842.pdf
        Self {
            a: 1.01605291,
            b: 0.492100,
            c: 0.0,
            d: 1.01605291,
            e: 0.0,
            animate: false,
        }
    }
}

impl Material2d for UafMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/uaf.wgsl".into()
    }
}

#[derive(Clone, Default, ShaderType)]
struct UafMaterialUniform {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub animate: f32,
}

impl AsBindGroupShaderType<UafMaterialUniform> for UafMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> UafMaterialUniform {
        UafMaterialUniform {
            a: self.a,
            b: self.b,
            c: self.c,
            d: self.d,
            e: self.e,
            animate: if self.animate { 1.0 } else { 0.0 },
        }
    }
}


pub fn main() {
    setup_hooks();
    example_app();
}

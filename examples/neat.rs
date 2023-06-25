use bevy::{
    prelude::*,
    render::{
        extract_resource::{
            ExtractResource,
            ExtractResourcePlugin,
        },
        render_asset::RenderAssets,
        render_graph::{
            self,
            RenderGraph,
        },
        renderer::{
            RenderContext,
            RenderDevice,
        },
        render_resource::{
            BindGroup,
            BindGroupDescriptor,
            BindGroupEntry,
            BindGroupLayout,
            BindGroupLayoutDescriptor,
            BindGroupLayoutEntry,
            BindingResource,
            BindingType,
            CachedComputePipelineId,
            CachedPipelineState,
            ComputePassDescriptor,
            ComputePipelineDescriptor,
            Extent3d,
            PipelineCache,
            ShaderStages,
            StorageTextureAccess,
            TextureDimension,
            TextureFormat,
            TextureUsages,
            TextureViewDimension,
        },
        RenderApp,
        RenderSet,
    },
};

use rusty_automata::{
    RustyAutomataApp,
    utils::setup_hooks,
};

use std::borrow::Cow;


const WORKGROUP_SIZE: u32 = 8;


fn example_app() {
    App::new()
        .add_plugin(RustyAutomataApp::default())
        .add_plugin(NeatComputePlugin)
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

    // TODO: initialize state, edges, activation function, and input batch



    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.resolution.width() as f32, window.resolution.height() as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(NeatImage {
        handle: image,
        size: (size.width, size.height),
    });
}


pub struct NeatComputePlugin;

impl Plugin for NeatComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractResourcePlugin::<NeatImage>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<NeatPipeline>()
            .add_system(queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("neat", NeatNode::default());
        render_graph.add_node_edge(
            "neat",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }
}

#[derive(Resource, Clone, ExtractResource)]
struct NeatImage {
    handle: Handle<Image>,
    size: (u32, u32),
}

#[derive(Resource, Clone, ExtractResource)]
struct NeatEdges {
    handle: Handle<Image>,
    max_edges: u32,
}

#[derive(Resource, Clone, Deref, ExtractResource)]
struct NeatActivationFunctions(Handle<Image>);

#[derive(Resource, Clone, ExtractResource)]
struct NeatIO {
    handle: Handle<Image>,
    size: (u32, u32),
    current_index: u32,
}

#[derive(Resource)]
struct NeatImageBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    mut pipeline: ResMut<NeatPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<NeatImage>,
    render_device: Res<RenderDevice>,
) {
    let view = &gpu_images[&game_of_life_image.handle];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });
    commands.insert_resource(NeatImageBindGroup(bind_group));

    pipeline.size = game_of_life_image.size;
}

#[derive(Resource)]
pub struct NeatPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    size: (u32, u32),
}

impl FromWorld for NeatPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });
        let shader = world
            .resource::<AssetServer>()
            .load("shaders/neat.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        NeatPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
            size: (0, 0),
        }
    }
}

enum NeatState {
    Loading,
    Init,
    Update,
}

struct NeatNode {
    state: NeatState,
}

impl Default for NeatNode {
    fn default() -> Self {
        Self {
            state: NeatState::Loading,
        }
    }
}

impl render_graph::Node for NeatNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<NeatPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            NeatState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = NeatState::Init;
                }
            }
            NeatState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = NeatState::Update;
                }
            }
            NeatState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let neat_bind_group = world.resource::<NeatImageBindGroup>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<NeatPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, &neat_bind_group.0, &[]);

        match self.state {
            NeatState::Loading => {}
            NeatState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(pipeline.size.0 / WORKGROUP_SIZE, pipeline.size.1 / WORKGROUP_SIZE, 1);
            }
            NeatState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(pipeline.size.0 / WORKGROUP_SIZE, pipeline.size.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}

pub fn main() {
    setup_hooks();
    example_app();
}

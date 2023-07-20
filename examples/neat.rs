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
            RenderQueue,
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
            BufferBindingType,
            CachedComputePipelineId,
            CachedPipelineState,
            ComputePassDescriptor,
            ComputePipelineDescriptor,
            Extent3d,
            PipelineCache,
            ShaderStages,
            ShaderType,
            StorageTextureAccess,
            TextureDimension,
            TextureFormat,
            TextureUsages,
            TextureViewDimension,
            UniformBuffer,
        },
        Render,
        RenderApp,
        RenderSet,
    },
};

use num_format::{Locale, ToFormattedString};

use rusty_automata::{
    RustyAutomataApp,
    noise::NoisePlugin,
    uaf::UafPlugin,
    utils::setup_hooks,
};

use std::borrow::Cow;


const WORKGROUP_SIZE: u32 = 4;


fn example_app() {
    App::new()
        .add_plugins((
            RustyAutomataApp::default(),
            NeatComputePlugin,
            NoisePlugin,
            UafPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}


fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = windows.single();
    let field_size = Extent3d {
        width: window.resolution.width() as u32,
        height: window.resolution.height() as u32,
        depth_or_array_layers: 1,
    };

    let mut activations = Image::new_fill(
        field_size,
        TextureDimension::D2,
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        TextureFormat::Rgba32Float,
    );
    activations.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let activations = images.add(activations);

    let mut nodes = Image::new_fill(
        field_size,
        TextureDimension::D2,
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        TextureFormat::Rgba32Float,
    );
    nodes.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let nodes = images.add(nodes);


    let edge_neighborhood: u32 = 5;

    // 2D to assist cache locality
    let edges_size = Extent3d {
        width: field_size.width * edge_neighborhood,
        height: field_size.height * edge_neighborhood,
        depth_or_array_layers: 1,
    };

    let mut edges: Image = Image::new_fill(
        edges_size,
        TextureDimension::D2,
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        TextureFormat::Rgba32Float,
    );
    edges.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let edges = images.add(edges);

    commands.insert_resource(NeatField {
        activations: activations.clone(),
        edges: edges.clone(),
        nodes: nodes.clone(),
        edge_neighborhood,
        size: (field_size.width, field_size.height),
        max_radius: 35.0,
        max_edge_weight: 12.0,
        manual_init: false,
    });

    println!("field_size: {:?}x{:?}", field_size.width, field_size.height);
    let parameters = field_size.width * field_size.height * 7 + edge_neighborhood * edge_neighborhood * 4;
    println!("parameters: {}", parameters.to_formatted_string(&Locale::en));

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(window.resolution.width() as f32, window.resolution.height() as f32)),
            ..default()
        },
        texture: nodes,
        ..default()
    });
    commands.spawn(Camera2dBundle::default());
}


pub struct NeatComputePlugin;

impl Plugin for NeatComputePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<NeatField>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                prepare_neat_uniforms.in_set(RenderSet::Prepare),
                queue_bind_group.in_set(RenderSet::Queue),
            )
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        // TODO: add cli args to NeatNode::default()
        render_graph.add_node("neat", NeatNode::default());
        render_graph.add_node_edge(
            "neat",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<NeatPipeline>();
        render_app.init_resource::<NeatUniformBuffer>();
    }
}

#[derive(Resource, Clone, ExtractResource)]
struct NeatField {
    activations: Handle<Image>,
    edges: Handle<Image>,
    nodes: Handle<Image>,
    edge_neighborhood: u32,
    size: (u32, u32),
    max_radius: f32,
    max_edge_weight: f32,
    manual_init: bool,
}

// #[derive(Resource, Clone, ExtractResource)]
// struct NeatIO {
//     handle: Handle<Image>,
//     size: (u32, u32),
//     current_index: u32,
// }


#[derive(Clone, Default, ShaderType)]
pub struct NeatUniform {
    edge_neighborhood: u32,
    max_radius: f32,
    max_edge_weight: f32,
    width: u32,
    height: u32,
    // TODO: add time uniform (or random seed)
}

#[derive(Resource, Default)]
pub struct NeatUniformBuffer {
    pub buffer: UniformBuffer<NeatUniform>,
}

fn prepare_neat_uniforms(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut uniform_buffer: ResMut<NeatUniformBuffer>,
    neat_field: Res<NeatField>,
) {
    let buffer = uniform_buffer.buffer.get_mut();

    buffer.edge_neighborhood = neat_field.edge_neighborhood;
    buffer.max_radius = neat_field.max_radius;
    buffer.max_edge_weight = neat_field.max_edge_weight;
    buffer.width = neat_field.size.0;
    buffer.height = neat_field.size.1;

    uniform_buffer.buffer.write_buffer(&render_device, &render_queue);
}


#[derive(Resource)]
struct NeatBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    mut pipeline: ResMut<NeatPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    neat_field: Res<NeatField>,
    render_device: Res<RenderDevice>,
    uniform_buffer: ResMut<NeatUniformBuffer>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(
                    &gpu_images[&neat_field.activations].texture_view
                ),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(
                    &gpu_images[&neat_field.edges].texture_view
                ),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(
                    &gpu_images[&neat_field.nodes].texture_view
                ),
            },
            BindGroupEntry {
                binding: 3,
                resource: uniform_buffer.buffer.binding().unwrap(),
            }
        ],
    });

    commands.insert_resource(NeatBindGroup(bind_group));

    pipeline.size = neat_field.size;
    pipeline.manual_init = neat_field.manual_init;
}

#[derive(Resource)]
pub struct NeatPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    size: (u32, u32),
    manual_init: bool,
}

impl FromWorld for NeatPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::ReadWrite,
                                format: TextureFormat::Rgba32Float,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 3,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
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
            manual_init: false,
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

        if pipeline.manual_init {
            self.state = NeatState::Init;
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let neat_bind_group = world.resource::<NeatBindGroup>();
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
